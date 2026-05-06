//! HTML report builder - generates the full interactive `PopQC` report

#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_lines)]

use chrono::Utc;
use popqc_core::PopQCError;
use popqc_core::config::PopQCConfig;
use popqc_core::error::Result;
use popqc_core::model::CohortFrame;

pub fn build_html(frame: &CohortFrame, config: &PopQCConfig) -> Result<String> {
    let data_json = serde_json::to_string(&frame.to_json_records())
        .map_err(|e| PopQCError::Report(e.to_string()))?;
    let columns = build_column_list(frame);
    let cols_json =
        serde_json::to_string(&columns).map_err(|e| PopQCError::Report(e.to_string()))?;
    let plot_cols: Vec<&str> = frame.metric_ids().iter().map(String::as_str).collect();
    let pcols_json =
        serde_json::to_string(&plot_cols).map_err(|e| PopQCError::Report(e.to_string()))?;
    let meta_cols = build_metadata_columns(frame);
    let meta_cols_json =
        serde_json::to_string(&meta_cols).map_err(|e| PopQCError::Report(e.to_string()))?;
    let n = frame.num_samples();
    let n_metrics = frame.num_metrics();
    let n_meta = meta_cols.len();
    let title = &config.report.title;
    let project_name = &config.project.name;
    let pipeline = &config.project.pipeline;
    let genome = &config.project.genome;
    let annotation = &config.project.annotation;
    let generated_at = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let html = format!(
        r#"{head}
{body_open}
{fullscreen_modal}
{header}
{container_open}
{summary_card}
{tabs_nav}
{tab1_table}
{tab2_explore}
{tab3_pca}
{tab4_compare}
{container_close}
{footer}
{javascript}
</body>
</html>"#,
        head = build_head(title),
        body_open = "<body>",
        fullscreen_modal = FULLSCREEN_MODAL,
        header = build_header(project_name, pipeline, genome, annotation, &generated_at),
        container_open = r#"<div class="container">"#,
        summary_card = build_summary_card(n, n_metrics, n_meta),
        tabs_nav = TABS_NAV,
        tab1_table = TAB1_TABLE,
        tab2_explore = TAB2_EXPLORE,
        tab3_pca = TAB3_PCA,
        tab4_compare = TAB4_COMPARE,
        container_close = "</div>",
        footer = build_footer(&generated_at),
        javascript = build_javascript(&data_json, &cols_json, &pcols_json, &meta_cols_json),
    );

    Ok(html)
}

fn build_column_list(frame: &CohortFrame) -> Vec<String> {
    let mut cols = vec!["Sample".to_string()];
    for key in frame.metadata_keys() {
        cols.push(key.clone());
    }
    for metric_id in frame.metric_ids() {
        cols.push(metric_id.clone());
    }
    cols
}

fn build_metadata_columns(frame: &CohortFrame) -> Vec<String> {
    frame.metadata_keys().to_vec()
}

fn build_head(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<script src="https://cdn.plot.ly/plotly-3.4.0.min.js" charset="utf-8"></script>
{css}
</head>"#,
        title = title,
        css = CSS_STYLES
    )
}

fn build_header(name: &str, pipeline: &str, genome: &str, annotation: &str, date: &str) -> String {
    format!(
        r#"<div class="header">
<h1>{name}</h1>
<p class="sub">Population-level Quality Control Report</p>
<span class="badge">{pipeline} | {genome} | {annotation} | Generated: {date}</span>
</div>"#
    )
}

fn build_summary_card(n_samples: usize, n_metrics: usize, n_meta: usize) -> String {
    let meta_html = if n_meta > 0 {
        format!(
            r#"<div class="stat-box s-green"><div class="value">{n_meta}</div><div class="label">Metadata Fields</div></div>"#
        )
    } else {
        String::new()
    };
    format!(
        r#"<div class="card">
<h2>Cohort Summary</h2>
<div class="stats-grid">
<div class="stat-box s-purple"><div class="value">{n_samples}</div><div class="label">Total Samples</div></div>
<div class="stat-box s-blue"><div class="value">{n_metrics}</div><div class="label">QC Metrics</div></div>
{meta_html}
</div>
</div>"#
    )
}

fn build_footer(date: &str) -> String {
    format!(
        r#"<footer>
<div class="lab">PopQC</div>
<div style="margin-top:6px">Population-level Quality Control | Generated {date}</div>
<div style="margin-top:4px;font-size:.75em;color:#9ca3af">github.com/TNS-Schrauwen/popqc</div>
</footer>"#
    )
}

fn build_javascript(
    data_json: &str,
    cols_json: &str,
    pcols_json: &str,
    meta_cols_json: &str,
) -> String {
    format!(
        "<script>\n{js_core}\n</script>",
        js_core = build_js_core(data_json, cols_json, pcols_json, meta_cols_json)
    )
}

fn build_js_core(
    data_json: &str,
    cols_json: &str,
    pcols_json: &str,
    meta_cols_json: &str,
) -> String {
    format!(
        r#"
"use strict";
var D={data_json};
var C={cols_json};
var PC={pcols_json};
var META_COLS={meta_cols_json};
var sc=null,sa=true;
var sel=new Set();
var fd=D.slice();
var PAL=['#6366f1','#ec4899','#f59e0b','#10b981','#ef4444','#8b5cf6','#06b6d4','#84cc16','#f97316','#14b8a6'];
var lastFilterData={{}};

var QC_THRESHOLDS = {{
    'uniquely_mapped_percent': {{good_min:80, warn_min:60, direction:'higher'}},
    'uniquely_mapped': {{good_min:80, warn_min:60, direction:'higher'}},
    'percent_mapped': {{good_min:80, warn_min:60, direction:'higher'}},
    'mapped_passed_pct': {{good_min:80, warn_min:60, direction:'higher'}},
    'aligned': {{good_min:80, warn_min:60, direction:'higher'}},
    'rrna': {{good_max:10, warn_max:30, direction:'lower'}},
    'percent_rrna': {{good_max:10, warn_max:30, direction:'lower'}},
    'percent_gc': {{good_min:40, good_max:60, warn_min:30, warn_max:70, direction:'range'}},
    'gc': {{good_min:40, good_max:60, warn_min:30, warn_max:70, direction:'range'}},
    'median_tin': {{good_min:60, warn_min:40, direction:'higher'}},
    'tin': {{good_min:60, warn_min:40, direction:'higher'}},
    'percent_duplication': {{good_max:50, warn_max:70, direction:'lower'}},
    'percent_duplicates': {{good_max:50, warn_max:70, direction:'lower'}},
    'percent_assigned': {{good_min:50, warn_min:30, direction:'higher'}},
    'assigned': {{good_min:50, warn_min:30, direction:'higher'}},
    'total_sequences': {{good_min:30000000, warn_min:10000000, direction:'higher'}},
    'total_reads': {{good_min:30000000, warn_min:10000000, direction:'higher'}}
}};

var metaColorCache = {{}};
var metaColorIndex = {{}};
var META_PALETTES = {{
    'default': ['#6366f1','#ec4899','#f59e0b','#10b981','#ef4444','#8b5cf6','#06b6d4','#84cc16','#f97316','#14b8a6','#a855f7','#0ea5e9','#d946ef','#22c55e','#eab308'],
    'sex': {{'male':'#3b82f6','female':'#ec4899','m':'#3b82f6','f':'#ec4899','unknown':'#9ca3af'}},
    'gender': {{'male':'#3b82f6','female':'#ec4899','m':'#3b82f6','f':'#ec4899','unknown':'#9ca3af'}}
}};

var META_COL_IS_CATEGORICAL = {{}};
(function detectCategoricalCols() {{
    var MAX_UNIQUE_FOR_COLOR = 15;
    var MAX_RATIO_FOR_COLOR = 0.05;
    for (var mi = 0; mi < META_COLS.length; mi++) {{
        var col = META_COLS[mi];
        var uniqueVals = new Set();
        var allNumeric = true;
        var nonEmpty = 0;
        for (var i = 0; i < D.length; i++) {{
            var v = D[i][col];
            if (v != null && v !== '') {{
                uniqueVals.add(v);
                nonEmpty++;
                if (isNaN(Number(v))) allNumeric = false;
            }}
        }}
        var numUnique = uniqueVals.size;
        var ratio = nonEmpty > 0 ? numUnique / nonEmpty : 1;
        if (numUnique <= MAX_UNIQUE_FOR_COLOR && (!allNumeric || numUnique <= 5)) {{
            META_COL_IS_CATEGORICAL[col] = true;
        }} else if (ratio <= MAX_RATIO_FOR_COLOR && !allNumeric) {{
            META_COL_IS_CATEGORICAL[col] = true;
        }} else {{
            META_COL_IS_CATEGORICAL[col] = false;
        }}
    }}
}})();

function getMetaColor(col, val) {{
    if (!val || val === '') return null;
    if (!META_COL_IS_CATEGORICAL[col]) return null;
    var lowerCol = col.toLowerCase();
    var lowerVal = val.toLowerCase();
    if (lowerCol === 'sex' || lowerCol === 'gender') {{
        var sexPal = META_PALETTES['sex'];
        if (sexPal[lowerVal]) return sexPal[lowerVal];
        return '#9ca3af';
    }}
    var key = col + '::' + lowerVal;
    if (metaColorCache[key]) return metaColorCache[key];
    if (!metaColorIndex[col]) metaColorIndex[col] = 0;
    var palette = META_PALETTES['default'];
    var color = palette[metaColorIndex[col] % palette.length];
    metaColorIndex[col]++;
    metaColorCache[key] = color;
    return color;
}}

function getQcCellClass(colName, value) {{
    if (value === '' || value == null || isNaN(value)) return '';
    var v = Number(value);
    var lowerCol = colName.toLowerCase();
    for (var pattern in QC_THRESHOLDS) {{
        if (lowerCol.indexOf(pattern) >= 0) {{
            var t = QC_THRESHOLDS[pattern];
            if (t.direction === 'higher') {{
                if (v >= t.good_min) return 'cg';
                if (v >= t.warn_min) return 'cw';
                return 'cf';
            }} else if (t.direction === 'lower') {{
                if (v <= t.good_max) return 'cg';
                if (v <= t.warn_max) return 'cw';
                return 'cf';
            }} else if (t.direction === 'range') {{
                if (v >= t.good_min && v <= t.good_max) return 'cg';
                if (v >= t.warn_min && v <= t.warn_max) return 'cw';
                return 'cf';
            }}
        }}
    }}
    return '';
}}

/* ===== UTILITY ===== */
function isNumCol(c){{return PC.indexOf(c)>=0;}}
function isMetaCol(c){{return META_COLS.indexOf(c)>=0;}}
function fmtMeta(col, v) {{
    if (!v || v === '') return '';
    var color = getMetaColor(col, v);
    if (color) {{
        return '<span class="tag-meta" style="background:'+color+'15;color:'+color+';border-color:'+color+'40">'+v+'</span>';
    }}
    return '<span class="tag-meta-plain">'+v+'</span>';
}}

/* ===== TABLE ===== */
function renderT(){{
    var h=document.getElementById('th');
    var hhtml='<th style="width:30px"><input type="checkbox" onchange="ta(this)"></th>';
    for(var i=0;i<C.length;i++)hhtml+='<th onclick="st('+i+')">'+C[i]+'</th>';
    h.innerHTML=hhtml;
    var b=document.getElementById('tb');
    var bhtml='';
    for(var ri=0;ri<fd.length;ri++){{
        var row=fd[ri];
        var id=row.Sample||'';
        var s=sel.has(id)?'selected':'';
        var ch=sel.has(id)?'checked':'';
        bhtml+='<tr class="'+s+'" onclick="ts(\''+id.replace(/'/g,"\\'")+'\')">';
        bhtml+='<td><input type="checkbox" '+ch+' onclick="event.stopPropagation();ts(\''+id.replace(/'/g,"\\'")+'\')" ></td>';
        for(var ci=0;ci<C.length;ci++){{
            var c=C[ci],v=row[c];
            if(isMetaCol(c)){{
                bhtml+='<td>'+fmtMeta(c,v)+'</td>';
                continue;
            }}
            var cls = getQcCellClass(c, v);
            bhtml+='<td class="'+cls+'">'+(v!=null&&v!==''?v:'')+'</td>';
        }}
        bhtml+='</tr>';
    }}
    b.innerHTML=bhtml;
    document.getElementById('rc').textContent='Showing '+fd.length+' of '+D.length;
    document.getElementById('si').textContent=sel.size>0?'('+sel.size+' selected)':'';
}}
function st(i){{
    if(sc===i)sa=!sa;else{{sc=i;sa=true;}}
    var k=C[i];
    fd.sort(function(a,b){{
        var va=a[k],vb=b[k];
        if(va==null||va==='')return 1;
        if(vb==null||vb==='')return-1;
        if(!isNaN(va)&&!isNaN(vb))return sa?va-vb:vb-va;
        return sa?String(va).localeCompare(String(vb)):String(vb).localeCompare(String(va));
    }});
    renderT();
}}
function ft(){{
    var q=document.getElementById('sb').value.toLowerCase();
    var mf=document.getElementById('meta-filter');
    var mfVal=mf?mf.value:'';
    var mfCol=document.getElementById('meta-filter-col');
    var mfColVal=mfCol?mfCol.value:'';
    fd=D.filter(function(row){{
        var ms=!q||(row.Sample||'').toLowerCase().indexOf(q)>=0;
        var mm=true;
        if(mfColVal&&mfVal){{
            mm=(row[mfColVal]||'').toLowerCase()===mfVal.toLowerCase();
        }}
        return ms&&mm;
    }});
    renderT();
}}
function ts(id){{if(sel.has(id))sel.delete(id);else sel.add(id);renderT();}}
function ta(cb){{if(cb.checked)fd.forEach(function(r){{sel.add(r.Sample);}});else sel.clear();renderT();}}
function updateMetaFilterOptions(){{
    var col=document.getElementById('meta-filter-col').value;
    var select=document.getElementById('meta-filter');
    if(!col){{select.innerHTML='<option value="">All</option>';return;}}
    var vals=new Set();
    D.forEach(function(r){{if(r[col]&&r[col]!=='')vals.add(r[col]);}});
    var html='<option value="">All</option>';
    Array.from(vals).sort().forEach(function(v){{html+='<option value="'+v+'">'+v+'</option>';}});
    select.innerHTML=html;
}}

/* ===== CSV ===== */
function makeCsv(d,c){{
    var lines=[c.join(',')];
    for(var i=0;i<d.length;i++){{
        var row=[];
        for(var j=0;j<c.length;j++){{
            var v=d[i][c[j]];if(v==null)v='';
            row.push('"'+String(v).replace(/"/g,'""')+'"');
        }}
        lines.push(row.join(','));
    }}
    return lines.join('\n');
}}
function dlFile(content,name){{
    var b=new Blob([content],{{type:'text/csv'}});
    var a=document.createElement('a');a.href=URL.createObjectURL(b);a.download=name;a.click();
}}
function dlAll(){{dlFile(makeCsv(D,C),'popqc_all_samples.csv');}}
function dlSel(){{
    if(!sel.size){{alert('Select samples first');return;}}
    var s=D.filter(function(r){{return sel.has(r.Sample);}});
    dlFile(makeCsv(s,C),'popqc_selected_samples.csv');
}}

/* ===== FULLSCREEN ===== */
function openFullscreen(fi){{
    var fd2=lastFilterData[fi];if(!fd2)return;
    var overlay=document.getElementById('fs-overlay');
    var plotDiv=document.getElementById('fs-plot');
    overlay.style.display='flex';
    document.body.style.overflow='hidden';
    var metric=fd2.metric,minVal=fd2.minVal,maxVal=fd2.maxVal,nFlagged=fd2.nFlagged;
    var shapes=[];
    if(!isNaN(minVal))shapes.push({{type:'line',y0:minVal,y1:minVal,x0:0,x1:1,xref:'paper',line:{{color:'#ef4444',dash:'dash',width:2}}}});
    if(!isNaN(maxVal))shapes.push({{type:'line',y0:maxVal,y1:maxVal,x0:0,x1:1,xref:'paper',line:{{color:'#ef4444',dash:'dash',width:2}}}});
    var yMin=fd2.ys.length>0?Math.min.apply(null,fd2.ys):0;
    var yMax=fd2.ys.length>0?Math.max.apply(null,fd2.ys):1;
    var pad=(yMax-yMin)*0.1||1;
    var sMin=!isNaN(minVal)?minVal:yMin-pad;
    var sMax=!isNaN(maxVal)?maxVal:yMax+pad;
    shapes.push({{type:'rect',x0:0,x1:1,xref:'paper',y0:sMin,y1:sMax,fillcolor:'rgba(16,185,129,0.06)',line:{{width:0}}}});
    document.getElementById('fs-title').textContent=metric+' — '+nFlagged+' sample(s) flagged';
    Plotly.newPlot(plotDiv,[{{
        x:fd2.xs,y:fd2.ys,type:'scatter',mode:'markers',
        marker:{{color:fd2.colors,size:fd2.sizes.map(function(s){{return s*1.5;}}),line:{{width:1,color:'#fff'}}}},
        text:fd2.texts,hovertemplate:'%{{text}}<br><b>'+metric+'</b>: %{{y:.2f}}<extra></extra>'
    }}],{{
        title:{{text:'<b>'+metric+'</b> ('+nFlagged+' flagged)',font:{{size:18}}}},
        xaxis:{{tickangle:-45,tickfont:{{size:9}},automargin:true,title:'Samples'}},
        yaxis:{{title:{{text:metric,font:{{size:13}}}},automargin:true,gridcolor:'#e5e7eb'}},
        shapes:shapes,margin:{{l:80,r:40,t:70,b:140}},
        paper_bgcolor:'#fff',plot_bgcolor:'#fafbfc',hovermode:'closest'
    }},{{responsive:true,displaylogo:false}});
}}
function closeFullscreen(){{
    document.getElementById('fs-overlay').style.display='none';
    document.body.style.overflow='';
    Plotly.purge('fs-plot');
}}

/* ===== MULTI-FILTER EXPLORE ===== */
var multiFilters=[];
var multiFilterCounter=0;
function addFilter(){{
    multiFilterCounter++;
    multiFilters.push({{id:multiFilterCounter,metric:PC[0]||'',min:'',max:''}});
    renderMultiFilters();
}}
function removeFilter(id){{
    multiFilters=multiFilters.filter(function(f){{return f.id!==id;}});
    renderMultiFilters();
}}
function renderMultiFilters(){{
    var container=document.getElementById('multi-filter-list');
    var html='';
    for(var i=0;i<multiFilters.length;i++){{
        var f=multiFilters[i];
        html+='<div class="filter-row">';
        html+='<div class="filter-num">'+(i+1)+'</div>';
        html+='<select onchange="updateFilter('+f.id+',\'metric\',this.value)">';
        for(var j=0;j<PC.length;j++){{
            var s2=PC[j]===f.metric?' selected':'';
            html+='<option value="'+PC[j]+'"'+s2+'>'+PC[j]+'</option>';
        }}
        html+='</select>';
        html+='<div class="filter-input-group"><span class="filter-label">Min</span>';
        html+='<input type="number" step="any" placeholder="—" value="'+(f.min!==''?f.min:'')+'" onchange="updateFilter('+f.id+',\'min\',this.value)"></div>';
        html+='<div class="filter-input-group"><span class="filter-label">Max</span>';
        html+='<input type="number" step="any" placeholder="—" value="'+(f.max!==''?f.max:'')+'" onchange="updateFilter('+f.id+',\'max\',this.value)"></div>';
        html+='<button class="btn-remove" onclick="removeFilter('+f.id+')">&times;</button>';
        html+='</div>';
    }}
    container.innerHTML=html;
}}
function updateFilter(id,field,value){{
    for(var i=0;i<multiFilters.length;i++){{
        if(multiFilters[i].id===id){{multiFilters[i][field]=value;break;}}
    }}
}}
function runMultiFilter(){{
    if(multiFilters.length===0){{
        document.getElementById('explore-plots-area').innerHTML='<div class="empty-state">Add at least one filter to begin.</div>';
        document.getElementById('explore-summary').innerHTML='';return;
    }}
    var validFilters=multiFilters.filter(function(f){{return!isNaN(parseFloat(f.min))||!isNaN(parseFloat(f.max));}});
    if(validFilters.length===0){{
        document.getElementById('explore-plots-area').innerHTML='<div class="empty-state">Set min or max for at least one filter.</div>';
        document.getElementById('explore-summary').innerHTML='';return;
    }}
    var allFlagged=new Set();
    var sampleFailCounts={{}};
    var sampleFailDetails={{}};
    var flaggedPerFilter={{}};
    for(var fi=0;fi<validFilters.length;fi++){{
        var f=validFilters[fi],metric=f.metric;
        var minVal=parseFloat(f.min),maxVal=parseFloat(f.max);
        flaggedPerFilter[fi]=new Set();
        for(var i=0;i<D.length;i++){{
            var v=D[i][metric];
            if(v==null||v===''||isNaN(v))continue;
            var isFlagged=false;
            if(!isNaN(minVal)&&v<minVal)isFlagged=true;
            if(!isNaN(maxVal)&&v>maxVal)isFlagged=true;
            if(isFlagged){{
                flaggedPerFilter[fi].add(D[i].Sample);
                allFlagged.add(D[i].Sample);
                if(!sampleFailCounts[D[i].Sample])sampleFailCounts[D[i].Sample]=0;
                sampleFailCounts[D[i].Sample]++;
                if(!sampleFailDetails[D[i].Sample])sampleFailDetails[D[i].Sample]=[];
                sampleFailDetails[D[i].Sample].push({{metric:metric,value:v,min:f.min,max:f.max}});
            }}
        }}
    }}
    var plotContainer=document.getElementById('explore-plots-area');
    plotContainer.innerHTML='';
    lastFilterData={{}};
    var gridDiv=document.createElement('div');
    gridDiv.className='explore-grid';
    plotContainer.appendChild(gridDiv);
    for(var fi=0;fi<validFilters.length;fi++){{
        var f=validFilters[fi],metric=f.metric;
        var minVal=parseFloat(f.min),maxVal=parseFloat(f.max);
        var cardDiv=document.createElement('div');
        cardDiv.className='explore-plot-card';
        var headerDiv=document.createElement('div');
        headerDiv.className='explore-card-header';
        headerDiv.innerHTML='<span class="explore-card-title">'+metric+'</span><button class="fs-btn" onclick="openFullscreen('+fi+')" title="Fullscreen">&#x26F6;</button>';
        cardDiv.appendChild(headerDiv);
        var plotDiv=document.createElement('div');
        plotDiv.id='explore-plot-'+fi;
        cardDiv.appendChild(plotDiv);
        gridDiv.appendChild(cardDiv);
        var xs=[],ys=[],colors=[],sizes=[],texts=[];
        for(var i=0;i<D.length;i++){{
            var v=D[i][metric];
            if(v==null||v===''||isNaN(v))continue;
            xs.push(D[i].Sample);ys.push(v);
            texts.push(D[i].Sample);
            var flagged=false;
            if(!isNaN(minVal)&&v<minVal)flagged=true;
            if(!isNaN(maxVal)&&v>maxVal)flagged=true;
            colors.push(flagged?'#ef4444':'#6366f1');
            sizes.push(flagged?9:5);
        }}
        lastFilterData[fi]={{metric:metric,minVal:minVal,maxVal:maxVal,xs:xs,ys:ys,colors:colors,sizes:sizes,texts:texts,nFlagged:flaggedPerFilter[fi].size}};
        var shapes=[];
        if(!isNaN(minVal))shapes.push({{type:'line',y0:minVal,y1:minVal,x0:0,x1:1,xref:'paper',line:{{color:'#ef4444',dash:'dash',width:2}}}});
        if(!isNaN(maxVal))shapes.push({{type:'line',y0:maxVal,y1:maxVal,x0:0,x1:1,xref:'paper',line:{{color:'#ef4444',dash:'dash',width:2}}}});
        var yMin2=ys.length>0?Math.min.apply(null,ys):0,yMax2=ys.length>0?Math.max.apply(null,ys):1;
        var pad2=(yMax2-yMin2)*0.1||1;
        shapes.push({{type:'rect',x0:0,x1:1,xref:'paper',y0:(!isNaN(minVal)?minVal:yMin2-pad2),y1:(!isNaN(maxVal)?maxVal:yMax2+pad2),fillcolor:'rgba(16,185,129,0.06)',line:{{width:0}}}});
        Plotly.newPlot(plotDiv.id,[{{x:xs,y:ys,type:'scatter',mode:'markers',marker:{{color:colors,size:sizes,line:{{width:0.5,color:'#fff'}}}},text:texts,hovertemplate:'%{{text}}<br><b>'+metric+'</b>: %{{y:.2f}}<extra></extra>'}}],{{
            title:{{text:'<b>'+flaggedPerFilter[fi].size+' flagged</b>',font:{{size:11,color:flaggedPerFilter[fi].size>0?'#ef4444':'#10b981'}}}},
            xaxis:{{showticklabels:false,showgrid:false}},
            yaxis:{{title:{{text:metric,font:{{size:9}}}},automargin:true,gridcolor:'#f3f4f6',tickfont:{{size:9}}}},
            shapes:shapes,margin:{{l:55,r:10,t:30,b:15}},paper_bgcolor:'rgba(0,0,0,0)',plot_bgcolor:'#fafbfc',height:230
        }},{{responsive:true,displayModeBar:false}});
    }}
    var failedSorted=Object.keys(sampleFailCounts).sort(function(a,b){{return sampleFailCounts[b]-sampleFailCounts[a];}});
    var rhtml='<div class="explore-summary-header">';
    rhtml+='<div class="explore-stat-pill pill-red"><span class="pill-val">'+allFlagged.size+'</span><span class="pill-label">Flagged</span></div>';
    rhtml+='<div class="explore-stat-pill pill-blue"><span class="pill-val">'+validFilters.length+'</span><span class="pill-label">Filters</span></div>';
    rhtml+='<div class="explore-stat-pill pill-green"><span class="pill-val">'+(D.length-allFlagged.size)+'</span><span class="pill-label">Passed</span></div>';
    rhtml+='</div>';
    if(failedSorted.length>0){{
        rhtml+='<div class="explore-results-table-wrap"><table class="explore-results-table">';
        rhtml+='<thead><tr><th>Sample</th>';
        for(var mi=0;mi<META_COLS.length;mi++){{
            rhtml+='<th>'+META_COLS[mi]+'</th>';
        }}
        rhtml+='<th># Failed</th><th>Details</th></tr></thead><tbody>';
        for(var i=0;i<failedSorted.length;i++){{
            var samp=failedSorted[i];
            var sampData=D.find(function(r){{return r.Sample===samp;}});
            var details=sampleFailDetails[samp]||[];
            var chips='';
            for(var j=0;j<details.length;j++){{
                var dd=details[j];
                chips+='<span class="fail-chip">'+dd.metric+': <b>'+Number(dd.value).toFixed(2)+'</b>';
                if(dd.min!==''&&!isNaN(parseFloat(dd.min)))chips+=' <span class="fail-thresh">(min:'+dd.min+')</span>';
                if(dd.max!==''&&!isNaN(parseFloat(dd.max)))chips+=' <span class="fail-thresh">(max:'+dd.max+')</span>';
                chips+='</span>';
            }}
            var pct=(sampleFailCounts[samp]/validFilters.length*100);
            rhtml+='<tr><td class="samp-name">'+samp+'</td>';
            for(var mi=0;mi<META_COLS.length;mi++){{
                var metaVal=sampData?sampData[META_COLS[mi]]:'';
                rhtml+='<td>'+fmtMeta(META_COLS[mi],metaVal)+'</td>';
            }}
            rhtml+='<td><div class="fail-bar-wrap"><div class="fail-bar" style="width:'+pct+'%"></div><span class="fail-bar-text">'+sampleFailCounts[samp]+'/'+validFilters.length+'</span></div></td>';
            rhtml+='<td class="detail-cell">'+chips+'</td></tr>';
        }}
        rhtml+='</tbody></table></div>';
        if(META_COLS.length>0){{
            rhtml+='<div class="explore-meta-breakdown"><h3>Flagged Samples by Metadata</h3><div class="meta-breakdown-grid">';
            for(var mi=0;mi<META_COLS.length;mi++){{
                var col=META_COLS[mi];
                var counts={{}};
                for(var i=0;i<failedSorted.length;i++){{
                    var sd=D.find(function(r){{return r.Sample===failedSorted[i];}});
                    var val=sd?sd[col]:'';
                    if(!val)val='(empty)';
                    if(!counts[val])counts[val]=0;
                    counts[val]++;
                }}
                rhtml+='<div class="meta-breakdown-card"><div class="meta-breakdown-title">'+col+'</div>';
                var sorted=Object.keys(counts).sort(function(a,b){{return counts[b]-counts[a];}});
                for(var vi=0;vi<sorted.length;vi++){{
                    var val=sorted[vi];
                    var cnt=counts[val];
                    var pctMeta=(cnt/failedSorted.length*100).toFixed(0);
                    rhtml+='<div class="meta-breakdown-row">';
                    rhtml+='<span>'+fmtMeta(col,val==='(empty)'?'':val)+'</span>';
                    rhtml+='<span class="meta-breakdown-count">'+cnt+' ('+pctMeta+'%)</span>';
                    rhtml+='</div>';
                }}
                rhtml+='</div>';
            }}
            rhtml+='</div></div>';
        }}
    }}
    document.getElementById('explore-summary').innerHTML=rhtml;
}}
function selectMultiFilterFlagged(){{
    var validFilters=multiFilters.filter(function(f){{return!isNaN(parseFloat(f.min))||!isNaN(parseFloat(f.max));}});
    for(var fi=0;fi<validFilters.length;fi++){{
        var f=validFilters[fi],metric=f.metric,minVal=parseFloat(f.min),maxVal=parseFloat(f.max);
        for(var i=0;i<D.length;i++){{
            var v=D[i][metric];if(v==null||v===''||isNaN(v))continue;
            var flagged=false;
            if(!isNaN(minVal)&&v<minVal)flagged=true;
            if(!isNaN(maxVal)&&v>maxVal)flagged=true;
            if(flagged)sel.add(D[i].Sample);
        }}
    }}
    showTab('t1');renderT();
}}

function runPCA(){{
    var checkboxes=document.querySelectorAll('#pca-features input[type="checkbox"]:checked');
    var features=[];checkboxes.forEach(function(cb){{features.push(cb.value);}});
    if(features.length<2){{document.getElementById('pca-plot').innerHTML='<div class="empty-state">Select at least 2 features.</div>';return;}}
    var colorBy=document.getElementById('pca-color').value;
    var samples=[],matrix=[];
    for(var i=0;i<D.length;i++){{
        var row=[],valid=true;
        for(var j=0;j<features.length;j++){{var v=D[i][features[j]];if(v===''||v==null||isNaN(v)){{valid=false;break;}}row.push(Number(v));}}
        if(valid){{samples.push(i);matrix.push(row);}}
    }}
    if(matrix.length<3){{document.getElementById('pca-plot').innerHTML='<div class="empty-state">Not enough complete samples.</div>';return;}}
    var n=matrix.length,p=features.length;
    var means=new Array(p).fill(0),stds=new Array(p).fill(0);
    for(var j=0;j<p;j++){{for(var i=0;i<n;i++)means[j]+=matrix[i][j];means[j]/=n;}}
    for(var j=0;j<p;j++){{for(var i=0;i<n;i++)stds[j]+=(matrix[i][j]-means[j])*(matrix[i][j]-means[j]);stds[j]=Math.sqrt(stds[j]/(n-1));if(stds[j]===0)stds[j]=1;}}
    var Z=[];for(var i=0;i<n;i++){{var row=[];for(var j=0;j<p;j++)row.push((matrix[i][j]-means[j])/stds[j]);Z.push(row);}}
    var cov=[];for(var i=0;i<p;i++){{cov[i]=new Array(p).fill(0);for(var j=0;j<p;j++){{var s=0;for(var k=0;k<n;k++)s+=Z[k][i]*Z[k][j];cov[i][j]=s/(n-1);}}}}
    var A=cov.map(function(r){{return r.slice();}});var V=[];for(var i=0;i<p;i++){{V[i]=new Array(p).fill(0);V[i][i]=1;}}
    for(var iter=0;iter<100;iter++){{
        var mx=0,mi2=0,mj2=1;
        for(var i=0;i<p;i++)for(var j=i+1;j<p;j++)if(Math.abs(A[i][j])>mx){{mx=Math.abs(A[i][j]);mi2=i;mj2=j;}}
        if(mx<1e-10)break;
        var theta;if(Math.abs(A[mi2][mi2]-A[mj2][mj2])<1e-15)theta=Math.PI/4;
        else theta=0.5*Math.atan2(2*A[mi2][mj2],A[mi2][mi2]-A[mj2][mj2]);
        var co=Math.cos(theta),si2=Math.sin(theta);
        var nA=A.map(function(r){{return r.slice();}});
        for(var k=0;k<p;k++){{nA[k][mi2]=co*A[k][mi2]+si2*A[k][mj2];nA[k][mj2]=-si2*A[k][mi2]+co*A[k][mj2];}}
        for(var k=0;k<p;k++){{A[mi2][k]=co*nA[mi2][k]+si2*nA[mj2][k];A[mj2][k]=-si2*nA[mi2][k]+co*nA[mj2][k];}}
        A[mi2][mj2]=0;A[mj2][mi2]=0;
        var nV=V.map(function(r){{return r.slice();}});
        for(var k=0;k<p;k++){{nV[k][mi2]=co*V[k][mi2]+si2*V[k][mj2];nV[k][mj2]=-si2*V[k][mi2]+co*V[k][mj2];}}
        V=nV;
    }}
    var eigen=[];for(var i=0;i<p;i++)eigen.push({{val:A[i][i],idx:i}});
    eigen.sort(function(a,b){{return b.val-a.val;}});
    var totalVar=eigen.reduce(function(s,e){{return s+Math.max(0,e.val);}},0);
    var pc1=[],pc2=[];var ev1=eigen[0].idx,ev2=eigen.length>1?eigen[1].idx:0;
    for(var i=0;i<n;i++){{var v1=0,v2=0;for(var j=0;j<p;j++){{v1+=Z[i][j]*V[j][ev1];v2+=Z[i][j]*V[j][ev2];}}pc1.push(v1);pc2.push(v2);}}
    var var1=totalVar>0?(Math.max(0,eigen[0].val)/totalVar*100).toFixed(1):'0';
    var var2=totalVar>0&&eigen.length>1?(Math.max(0,eigen[1].val)/totalVar*100).toFixed(1):'0';
    var traces=[];
    if(colorBy){{
        var groups={{}};for(var i=0;i<samples.length;i++){{var g=D[samples[i]][colorBy]||'Unknown';if(!groups[g])groups[g]={{x:[],y:[],text:[]}};groups[g].x.push(pc1[i]);groups[g].y.push(pc2[i]);groups[g].text.push(D[samples[i]].Sample);}}
        var ci2=0,keys=Object.keys(groups).sort();
        for(var ki=0;ki<keys.length;ki++){{var gn=keys[ki],gd=groups[gn];traces.push({{x:gd.x,y:gd.y,mode:'markers',type:'scatter',name:gn+' ('+gd.x.length+')',text:gd.text,marker:{{color:PAL[ci2%PAL.length],size:8,opacity:0.8}},hovertemplate:'%{{text}}<br>PC1:%{{x:.2f}}<br>PC2:%{{y:.2f}}<extra>'+gn+'</extra>'}});ci2++;}}
    }}else{{
        var texts2=samples.map(function(i){{return D[i].Sample;}});
        traces.push({{x:pc1,y:pc2,mode:'markers',type:'scatter',text:texts2,marker:{{color:'#6366f1',size:8,opacity:0.8}},hovertemplate:'%{{text}}<br>PC1:%{{x:.2f}}<br>PC2:%{{y:.2f}}<extra></extra>'}});
    }}
    Plotly.newPlot('pca-plot',traces,{{
        title:{{text:'PCA ('+features.length+' features, '+samples.length+' samples)'}},
        xaxis:{{title:'PC1 ('+var1+'%)',automargin:true,zeroline:true}},
        yaxis:{{title:'PC2 ('+var2+'%)',automargin:true,zeroline:true}},
        margin:{{l:70,r:30,t:60,b:70}},paper_bgcolor:'rgba(0,0,0,0)',plot_bgcolor:'#fafbfc',height:550,
        legend:{{orientation:'h',y:-0.2,xanchor:'center',x:0.5}},hovermode:'closest'
    }},{{responsive:true}});
    document.getElementById('pca-info').innerHTML='<b>Variance:</b> PC1='+var1+'%, PC2='+var2+'% | <b>Samples:</b> '+samples.length+'/'+D.length;
}}
function pcaSelectAll(c){{document.querySelectorAll('#pca-features input[type="checkbox"]').forEach(function(cb){{cb.checked=c;}});}}

/* ===== COMPARE ===== */
var compareGroups={{}};var compareGroupCounter=0;
var CGC=['#6366f1','#ec4899','#f59e0b','#10b981','#ef4444','#8b5cf6','#06b6d4','#84cc16','#f97316','#14b8a6'];
function addCompareGroup(){{
    var name=document.getElementById('new-group-name').value.trim();
    if(!name){{alert('Enter a group name.');return;}}
    if(compareGroups[name]){{alert('Group exists.');return;}}
    compareGroupCounter++;
    compareGroups[name]={{samples:new Set(),color:CGC[(compareGroupCounter-1)%CGC.length]}};
    document.getElementById('new-group-name').value='';
    renderCompareGroups();
}}
function removeCompareGroup(name){{delete compareGroups[name];renderCompareGroups();}}
function renderCompareGroups(){{
    var container=document.getElementById('compare-groups-list');
    var groupNames=Object.keys(compareGroups);
    if(groupNames.length===0){{container.innerHTML='<div class="empty-state">Create a group to get started.</div>';return;}}
    var html='';
    for(var gi=0;gi<groupNames.length;gi++){{
        var gn=groupNames[gi],grp=compareGroups[gn],arr=Array.from(grp.samples);
        html+='<div class="compare-group-card" style="border-left:4px solid '+grp.color+'">';
        html+='<div class="compare-group-header"><span style="color:'+grp.color+';font-weight:700">'+gn+' ('+arr.length+')</span>';
        html+='<button class="btn-remove" onclick="removeCompareGroup(\''+gn.replace(/'/g,"\\'")+'\')">&times;</button></div>';
        html+='<div class="ms-dropdown-wrap"><div class="ms-dropdown-trigger" onclick="toggleMsDropdown('+gi+')">';
        html+='<span class="ms-placeholder">'+(arr.length>0?arr.length+' selected':'Click to select...')+'</span><span class="ms-arrow">&#9662;</span></div>';
        html+='<div class="ms-dropdown-panel" id="ms-panel-'+gi+'">';
        html+='<div class="ms-search-box"><input type="text" placeholder="Search..." oninput="filterMsDropdown('+gi+',this.value)"></div>';
        html+='<div class="ms-options" id="ms-options-'+gi+'">';
        for(var i=0;i<D.length;i++){{
            var chk=grp.samples.has(D[i].Sample);
            html+='<div class="ms-option '+(chk?'ms-checked':'')+'" onclick="toggleMsSample(\''+gn.replace(/'/g,"\\'")+'\',\''+D[i].Sample.replace(/'/g,"\\'")+'\','+gi+')" data-sample="'+D[i].Sample.toLowerCase()+'">';
            html+='<span class="ms-check">'+(chk?'&#10003;':'')+'</span><span class="ms-sample-name">'+D[i].Sample+'</span></div>';
        }}
        html+='</div></div></div>';
        if(arr.length>0){{
            html+='<div class="compare-sample-tags">';
            for(var si=0;si<arr.length;si++)html+='<span class="compare-sample-tag" style="border-color:'+grp.color+'">'+arr[si]+'<span class="compare-tag-remove" onclick="removeSampleFromGroup(\''+gn.replace(/'/g,"\\'")+'\',\''+arr[si].replace(/'/g,"\\'")+'\')\">&times;</span></span>';
            html+='</div>';
        }}
        html+='</div>';
    }}
    container.innerHTML=html;
}}
function toggleMsDropdown(gi){{document.getElementById('ms-panel-'+gi).classList.toggle('ms-open');}}
function filterMsDropdown(gi,q){{var opts=document.querySelectorAll('#ms-options-'+gi+' .ms-option');var ql=q.toLowerCase();opts.forEach(function(o){{o.style.display=(!ql||o.getAttribute('data-sample').indexOf(ql)>=0)?'':'none';}});}}
function toggleMsSample(gn,sample,gi){{var grp=compareGroups[gn];if(grp.samples.has(sample))grp.samples.delete(sample);else grp.samples.add(sample);renderCompareGroups();setTimeout(function(){{var p=document.getElementById('ms-panel-'+gi);if(p)p.classList.add('ms-open');}},50);}}
function removeSampleFromGroup(gn,s){{compareGroups[gn].samples.delete(s);renderCompareGroups();}}
function runCompare(){{
    var groupNames=Object.keys(compareGroups);
    if(groupNames.length===0){{alert('Define groups first.');return;}}
    var metric=document.getElementById('compare-metric').value;
    var mode=document.getElementById('compare-mode').value;
    var traces=[];
    for(var gi=0;gi<groupNames.length;gi++){{
        var gn=groupNames[gi],grp=compareGroups[gn],arr=Array.from(grp.samples);
        var vals=[],names=[];
        for(var si=0;si<arr.length;si++){{var sd=D.find(function(r){{return r.Sample===arr[si];}});if(sd){{var v=sd[metric];if(v!=null&&v!==''&&!isNaN(v)){{vals.push(Number(v));names.push(arr[si]);}}}}}}
        if(mode==='group'){{
            traces.push({{y:vals,type:'box',name:gn+' (n='+vals.length+')',marker:{{color:grp.color}},text:names,boxpoints:'all',jitter:0.3,pointpos:-1.5}});
        }}else{{
            traces.push({{x:names,y:vals,type:'bar',name:gn,marker:{{color:grp.color,opacity:0.85}}}});
        }}
    }}
    Plotly.newPlot('compare-plot-area',traces,{{
        title:{{text:(mode==='group'?'Group':'Sample')+' Comparison: '+metric}},
        yaxis:{{title:metric,automargin:true}},
        xaxis:{{tickangle:-45,automargin:true,tickfont:{{size:9}}}},
        barmode:'group',margin:{{l:70,r:30,t:60,b:120}},paper_bgcolor:'rgba(0,0,0,0)',plot_bgcolor:'#fafbfc',height:500
    }},{{responsive:true}});
}}

/* ===== TABS ===== */
function showTab(id){{
    document.querySelectorAll('.tc').forEach(function(t){{t.classList.remove('active');}});
    document.querySelectorAll('.tabs button').forEach(function(b){{b.classList.remove('active');}});
    document.getElementById(id).classList.add('active');
    document.querySelectorAll('.tabs button').forEach(function(b){{if(b.getAttribute('onclick').indexOf(id)>=0)b.classList.add('active');}});
    if(id==='t2'){{if(multiFilters.length===0)addFilter();renderMultiFilters();}}
    if(id==='t4'){{renderCompareGroups();initCompareMetric();}}
}}
function initCompareMetric(){{
    var s=document.getElementById('compare-metric');
    if(s&&s.options.length<=1){{var h='';for(var i=0;i<PC.length;i++)h+='<option value="'+PC[i]+'">'+PC[i]+'</option>';s.innerHTML=h;}}
}}
function initSelectors(){{
    var h='';for(var i=0;i<PC.length;i++)h+='<label style="display:inline-flex;align-items:center;gap:4px;margin:3px 12px 3px 0;font-size:.85em;cursor:pointer"><input type="checkbox" value="'+PC[i]+'" checked> '+PC[i]+'</label>';
    document.getElementById('pca-features').innerHTML=h;
    var colorSel=document.getElementById('pca-color');
    var ch='<option value="">None</option>';
    for(var i=0;i<META_COLS.length;i++)ch+='<option value="'+META_COLS[i]+'">'+META_COLS[i]+'</option>';
    colorSel.innerHTML=ch;
    var mfCol=document.getElementById('meta-filter-col');
    if(mfCol){{
        var mh='<option value="">Filter by...</option>';
        for(var i=0;i<META_COLS.length;i++)mh+='<option value="'+META_COLS[i]+'">'+META_COLS[i]+'</option>';
        mfCol.innerHTML=mh;
    }}
}}
document.addEventListener('click',function(e){{document.querySelectorAll('.ms-dropdown-panel.ms-open').forEach(function(p){{var w=p.closest('.ms-dropdown-wrap');if(w&&!w.contains(e.target))p.classList.remove('ms-open');}});}});
document.addEventListener('keydown',function(e){{if(e.key==='Escape')closeFullscreen();}});
console.log('PopQC Report loaded. Samples:',D.length,'Metrics:',PC.length,'Metadata fields:',META_COLS.length);
renderT();
initSelectors();
"#,
        data_json = data_json,
        cols_json = cols_json,
        pcols_json = pcols_json,
        meta_cols_json = meta_cols_json,
    )
}

// ===== STATIC HTML FRAGMENTS =====

const FULLSCREEN_MODAL: &str = r#"<div class="fs-overlay" id="fs-overlay" onclick="if(event.target===this)closeFullscreen()">
<div class="fs-modal">
<div class="fs-header"><h3 id="fs-title">Metric</h3><button class="fs-close" onclick="closeFullscreen()">&times;</button></div>
<div class="fs-body"><div id="fs-plot" style="width:100%;height:100%"></div></div>
</div>
</div>"#;

const TABS_NAV: &str = r#"<div class="tabs">
<button class="active" onclick="showTab('t1')">Sample Table</button>
<button onclick="showTab('t2')">Explore</button>
<button onclick="showTab('t3')">PCA</button>
<button onclick="showTab('t4')">Compare</button>
</div>"#;

const TAB1_TABLE: &str = r#"<div class="tc active" id="t1">
<div class="card">
<h2>Per-Sample QC Metrics</h2>
<div class="controls">
<input type="text" id="sb" placeholder="Search samples..." oninput="ft()">
<select id="meta-filter-col" onchange="updateMetaFilterOptions();ft()"></select>
<select id="meta-filter" onchange="ft()"><option value="">All</option></select>
<button class="btn-p" onclick="dlAll()">Export All CSV</button>
<button class="btn-s" onclick="dlSel()">Export Selected</button>
<span id="rc" style="font-size:.85em;color:#6b7280"></span>
<span id="si" class="sel-count"></span>
</div>
<div id="tw"><table id="qt"><thead><tr id="th"></tr></thead><tbody id="tb"></tbody></table></div>
</div>
</div>"#;

const TAB2_EXPLORE: &str = r#"<div class="tc" id="t2">
<div class="card">
<h2>Multi-Filter QC Explore</h2>
<p style="margin-bottom:16px;color:#6b7280;font-size:.9em">
Add multiple QC metric filters. Click <strong>Run Filters</strong> to identify samples failing any threshold. Click <strong>&#x26F6;</strong> on any plot for fullscreen interactive view.
</p>
<div style="display:flex;gap:10px;margin-bottom:16px;flex-wrap:wrap;align-items:center">
<button class="btn-p" onclick="addFilter()">+ Add Filter</button>
<button class="btn-p" onclick="runMultiFilter()" style="background:#059669">&#9654; Run Filters</button>
<button class="btn-s" onclick="selectMultiFilterFlagged()">Select Flagged in Table</button>
</div>
<div id="multi-filter-list"></div>
<div id="explore-summary" style="margin-top:20px"></div>
<div id="explore-plots-area" style="margin-top:16px"></div>
</div>
</div>"#;

const TAB3_PCA: &str = r#"<div class="tc" id="t3">
<div class="card">
<h2>PCA of QC Metrics</h2>
<p style="margin-bottom:14px;color:#6b7280;font-size:.9em">
Select QC metrics for PCA. Computed in-browser via eigenvalue decomposition. Color by any available metadata field.
</p>
<div class="pctl">
<label>Color by:</label>
<select id="pca-color"><option value="">None</option></select>
<button class="btn-p" onclick="runPCA()">Run PCA</button>
<button class="btn-s" onclick="pcaSelectAll(true)">All</button>
<button class="btn-s" onclick="pcaSelectAll(false)">None</button>
</div>
<div class="feat-box" id="pca-features"></div>
<div id="pca-plot" style="width:100%;min-height:550px"></div>
<div id="pca-info" style="margin-top:10px;font-size:.85em;padding:10px;background:#f8fafc;border-radius:8px"></div>
</div>
</div>"#;

const TAB4_COMPARE: &str = r#"<div class="tc" id="t4">
<div class="card">
<h2>Group Comparison</h2>
<p style="margin-bottom:14px;color:#6b7280;font-size:.9em">
Define groups, select samples via dropdown with checkmarks, then compare any QC metric across groups.
</p>
<div style="display:flex;gap:10px;align-items:center;margin-bottom:16px;flex-wrap:wrap">
<input type="text" id="new-group-name" placeholder="Group name..." style="padding:8px 14px;border:1.5px solid #d1d5db;border-radius:10px;font-size:.9em;min-width:200px">
<button class="btn-p" onclick="addCompareGroup()">Create Group</button>
</div>
<div id="compare-groups-list"></div>
<div style="margin-top:16px;padding-top:16px;border-top:1px solid #e5e7eb">
<div class="pctl">
<label>Metric:</label><select id="compare-metric"></select>
<label>Mode:</label><select id="compare-mode"><option value="group">Group (Box)</option><option value="sample">Sample (Bar)</option></select>
<button class="btn-p" onclick="runCompare()">Compare</button>
</div>
</div>
<div id="compare-plot-area" style="width:100%;min-height:400px;margin-top:16px"></div>
</div>
</div>"#;

const CSS_STYLES: &str = r#"<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:'Inter',-apple-system,sans-serif;background:#f0f2f5;color:#1a1a2e;line-height:1.5}
.header{background:linear-gradient(135deg,#0f0c29,#302b63,#24243e);color:#fff;padding:32px 40px;text-align:center}
.header h1{font-size:2em;font-weight:800;margin-bottom:6px}
.header .sub{font-size:1em;opacity:.85}
.header .badge{display:inline-block;background:rgba(255,255,255,.15);border-radius:20px;padding:4px 14px;font-size:.8em;margin-top:10px}
.container{max-width:100%;padding:24px 28px}
.card{background:#fff;border-radius:12px;padding:24px;margin-bottom:20px;box-shadow:0 1px 3px rgba(0,0,0,.08);border:1px solid #e5e7eb}
.card h2{font-size:1.15em;margin-bottom:14px;color:#1e1b4b}
.stats-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:14px}
.stat-box{border-radius:10px;padding:16px;text-align:center}
.stat-box .value{font-size:2em;font-weight:800}
.stat-box .label{font-size:.78em;margin-top:4px;opacity:.8}
.s-purple{background:linear-gradient(135deg,#ede9fe,#ddd6fe);color:#5b21b6}
.s-blue{background:linear-gradient(135deg,#dbeafe,#bfdbfe);color:#1e40af}
.s-green{background:linear-gradient(135deg,#d1fae5,#a7f3d0);color:#065f46}
.tag-meta{display:inline-block;padding:2px 10px;border-radius:12px;font-size:.72em;font-weight:600;border:1px solid;white-space:nowrap}
.tag-meta-plain{display:inline-block;padding:2px 8px;font-size:.78em;font-weight:500;color:#374151;white-space:nowrap}
.controls{display:flex;gap:10px;margin-bottom:14px;flex-wrap:wrap;align-items:center}
.controls input,.controls select{padding:8px 14px;border:1px solid #d1d5db;border-radius:8px;font-size:.88em;background:#fff}
.controls input:focus,.controls select:focus{border-color:#6366f1;outline:none}
.controls input[type="text"]{min-width:220px}
.btn-p{padding:8px 18px;border:none;border-radius:8px;cursor:pointer;font-size:.88em;font-weight:600;background:#4f46e5;color:#fff}
.btn-p:hover{background:#4338ca}
.btn-s{padding:8px 18px;border:1px solid #d1d5db;border-radius:8px;cursor:pointer;font-size:.88em;font-weight:600;background:#f3f4f6;color:#374151}
.btn-s:hover{background:#e5e7eb}
.btn-remove{background:#fee2e2;color:#991b1b;border:1px solid #fca5a5;border-radius:6px;width:28px;height:28px;display:inline-flex;align-items:center;justify-content:center;cursor:pointer;font-size:1.1em;font-weight:700;flex-shrink:0}
#tw{overflow-x:auto;max-height:650px;overflow-y:auto;border-radius:8px;border:1px solid #e5e7eb}
#qt{border-collapse:collapse;width:100%;font-size:.8em}
#qt th{position:sticky;top:0;background:#1e1b4b;color:#fff;padding:10px 12px;text-align:left;cursor:pointer;white-space:nowrap;z-index:2;font-weight:600;font-size:.85em}
#qt th:hover{background:#312e81}
#qt td{padding:7px 12px;border-bottom:1px solid #f3f4f6;white-space:nowrap}
#qt tr:nth-child(even){background:#fafbfc}
#qt tr:hover{background:#eef2ff}
#qt tr.selected{background:#c7d2fe!important}
.cg{background:#d1fae5;color:#065f46;font-weight:500}
.cw{background:#fef3c7;color:#92400e;font-weight:500}
.cf{background:#fee2e2;color:#991b1b;font-weight:500}
.tabs{display:flex;gap:4px;margin-bottom:20px;background:#f3f4f6;padding:4px;border-radius:10px;flex-wrap:wrap}
.tabs button{padding:10px 24px;border:none;background:transparent;border-radius:8px;cursor:pointer;font-size:.9em;font-weight:600;color:#6b7280}
.tabs button.active{background:#fff;color:#4f46e5;box-shadow:0 1px 3px rgba(0,0,0,.1)}
.tc{display:none}.tc.active{display:block}
.pctl{display:flex;gap:12px;margin-bottom:14px;flex-wrap:wrap;align-items:center}
.pctl label{font-weight:600;font-size:.88em;color:#374151}
.pctl select,.pctl input{padding:8px 12px;border:1px solid #d1d5db;border-radius:8px}
.sel-count{font-weight:700;color:#4f46e5}
.feat-box{max-height:180px;overflow-y:auto;border:1px solid #e5e7eb;border-radius:8px;padding:10px;background:#fafbfc;margin-bottom:12px}
.empty-state{padding:40px;text-align:center;color:#6b7280;font-size:.95em}
.filter-row{display:flex;gap:10px;align-items:center;margin-bottom:8px;padding:12px 16px;background:#fafbfc;border:1px solid #e5e7eb;border-radius:10px;flex-wrap:wrap;transition:all .2s}
.filter-row:hover{border-color:#c7d2fe;background:#f5f3ff}
.filter-row select,.filter-row input{padding:7px 12px;border:1px solid #d1d5db;border-radius:8px;font-size:.85em}
.filter-row select{min-width:180px;background:#fff}
.filter-row input{width:90px}
.filter-num{width:24px;height:24px;background:#6366f1;color:#fff;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:.75em;font-weight:700;flex-shrink:0}
.filter-input-group{display:flex;align-items:center;gap:4px}
.filter-label{font-size:.78em;font-weight:600;color:#6b7280}
.explore-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(380px,1fr));gap:16px;margin-bottom:20px}
.explore-plot-card{background:#fff;border:1px solid #e5e7eb;border-radius:10px;overflow:hidden;box-shadow:0 1px 2px rgba(0,0,0,.04);transition:box-shadow .2s}
.explore-plot-card:hover{box-shadow:0 4px 12px rgba(0,0,0,.08)}
.explore-card-header{display:flex;align-items:center;justify-content:space-between;padding:8px 12px;background:#f8fafc;border-bottom:1px solid #f3f4f6}
.explore-card-title{font-size:.82em;font-weight:700;color:#1e1b4b;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.fs-btn{background:none;border:1.5px solid #d1d5db;border-radius:6px;width:30px;height:30px;display:flex;align-items:center;justify-content:center;cursor:pointer;font-size:1em;color:#6b7280;flex-shrink:0;transition:all .15s}
.fs-btn:hover{background:#eef2ff;border-color:#6366f1;color:#4f46e5}
.explore-summary-header{display:flex;gap:12px;flex-wrap:wrap;margin-bottom:16px}
.explore-stat-pill{display:flex;flex-direction:column;align-items:center;padding:10px 20px;border-radius:10px;min-width:90px}
.pill-val{font-size:1.5em;font-weight:800}.pill-label{font-size:.72em;font-weight:600;opacity:.8}
.pill-red{background:#fee2e2;color:#991b1b}
.pill-blue{background:#dbeafe;color:#1e40af}
.pill-green{background:#d1fae5;color:#065f46}
.explore-results-table-wrap{max-height:500px;overflow:auto;border:1px solid #e5e7eb;border-radius:10px;background:#fff}
.explore-results-table{width:100%;border-collapse:collapse;font-size:.82em}
.explore-results-table thead th{position:sticky;top:0;background:#f8fafc;padding:10px 12px;text-align:left;font-weight:700;border-bottom:2px solid #e5e7eb;z-index:1}
.explore-results-table td{padding:8px 12px;border-bottom:1px solid #f3f4f6;vertical-align:top}
.explore-results-table tr:hover{background:#fafbfc}
.samp-name{font-weight:700;color:#1e1b4b;white-space:nowrap}
.fail-chip{display:inline-block;margin:2px 4px 2px 0;padding:3px 8px;background:#fef2f2;border:1px solid #fecaca;border-radius:6px;font-size:.85em;white-space:nowrap}
.fail-thresh{color:#6b7280;font-size:.85em}
.fail-bar-wrap{display:flex;align-items:center;gap:6px;min-width:100px}
.fail-bar{height:6px;background:linear-gradient(90deg,#f87171,#ef4444);border-radius:3px;min-width:4px}
.fail-bar-text{font-size:.8em;font-weight:700;color:#991b1b}
.detail-cell{max-width:400px;overflow-x:auto}
.explore-meta-breakdown{margin-top:20px;padding:16px;background:#f8fafc;border:1px solid #e5e7eb;border-radius:10px}
.explore-meta-breakdown h3{font-size:.95em;color:#1e1b4b;margin-bottom:12px}
.meta-breakdown-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:12px}
.meta-breakdown-card{background:#fff;border:1px solid #e5e7eb;border-radius:8px;padding:12px}
.meta-breakdown-title{font-size:.82em;font-weight:700;color:#4b5563;margin-bottom:8px;text-transform:capitalize}
.meta-breakdown-row{display:flex;justify-content:space-between;align-items:center;padding:3px 0;font-size:.82em}
.meta-breakdown-count{font-weight:600;color:#6b7280;font-size:.8em}
.fs-overlay{display:none;position:fixed;top:0;left:0;width:100vw;height:100vh;background:rgba(0,0,0,.5);backdrop-filter:blur(4px);z-index:9999;align-items:center;justify-content:center;padding:24px}
.fs-modal{background:#fff;border-radius:16px;width:calc(100vw - 48px);height:calc(100vh - 48px);max-width:1600px;display:flex;flex-direction:column;box-shadow:0 20px 60px rgba(0,0,0,.3);overflow:hidden}
.fs-header{display:flex;align-items:center;justify-content:space-between;padding:16px 24px;border-bottom:1px solid #e5e7eb;background:#f8fafc;flex-shrink:0}
.fs-header h3{font-size:1.1em;color:#1e1b4b;font-weight:700}
.fs-close{background:none;border:1.5px solid #d1d5db;border-radius:8px;width:36px;height:36px;display:flex;align-items:center;justify-content:center;cursor:pointer;font-size:1.3em;color:#6b7280}
.fs-close:hover{background:#fee2e2;border-color:#fca5a5;color:#991b1b}
.fs-body{flex:1;padding:16px;overflow:hidden}
.compare-group-card{background:#fafbfc;border:1px solid #e5e7eb;border-radius:10px;padding:16px;margin-bottom:12px}
.compare-group-header{display:flex;justify-content:space-between;align-items:center;margin-bottom:12px}
.compare-sample-tags{display:flex;flex-wrap:wrap;gap:6px;margin-top:10px}
.compare-sample-tag{display:inline-flex;align-items:center;gap:4px;padding:4px 10px;background:#fff;border:1.5px solid #d1d5db;border-radius:16px;font-size:.78em;font-weight:500}
.compare-tag-remove{cursor:pointer;color:#ef4444;font-weight:700;font-size:1.1em;margin-left:4px}
.ms-dropdown-wrap{position:relative;width:100%}
.ms-dropdown-trigger{display:flex;align-items:center;justify-content:space-between;padding:10px 14px;border:1.5px solid #d1d5db;border-radius:10px;cursor:pointer;background:#fff;transition:all .2s}
.ms-dropdown-trigger:hover{border-color:#6366f1}
.ms-placeholder{font-size:.88em;color:#4b5563}
.ms-arrow{font-size:.8em;color:#6b7280}
.ms-dropdown-panel{display:none;position:absolute;top:calc(100% + 4px);left:0;right:0;background:#fff;border:1.5px solid #e5e7eb;border-radius:10px;box-shadow:0 10px 40px rgba(0,0,0,.12);z-index:100;max-height:300px;overflow:hidden}
.ms-dropdown-panel.ms-open{display:block}
.ms-search-box{padding:10px 12px;border-bottom:1px solid #f3f4f6}
.ms-search-box input{width:100%;padding:8px 12px;border:1px solid #e5e7eb;border-radius:8px;font-size:.88em;outline:none}
.ms-options{max-height:220px;overflow-y:auto}
.ms-option{display:flex;align-items:center;gap:10px;padding:8px 14px;cursor:pointer;transition:background .15s;font-size:.88em}
.ms-option:hover{background:#f5f3ff}
.ms-option.ms-checked{background:#eef2ff}
.ms-check{width:18px;height:18px;border:2px solid #d1d5db;border-radius:4px;display:flex;align-items:center;justify-content:center;font-size:.7em;color:#4f46e5;font-weight:700;flex-shrink:0}
.ms-checked .ms-check{background:#eef2ff;border-color:#6366f1}
.ms-sample-name{font-weight:500;color:#1e1b4b}
footer{text-align:center;padding:28px;color:#6b7280;font-size:.82em;border-top:1px solid #e5e7eb;margin-top:20px;background:#fff}
footer .lab{font-weight:700;color:#4f46e5;font-size:1.1em}
</style>"#;
