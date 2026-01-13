use minijinja::{Environment, context};
use crate::search::SearchResult;

pub fn generate_html_report(query: &str, results: &[SearchResult]) -> String {
    let mut env = Environment::new();
    
    const TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Code-RAG Search Results</title>
    <style>
        body { font-family: -apple-system, system-ui, sans-serif; max_width: 800px; margin: 0 auto; padding: 20px; background: #f4f4f9; }
        .header { background: #333; color: #fff; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .result-card { background: #fff; border: 1px solid #ddd; border-radius: 8px; margin-bottom: 20px; padding: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.05); }
        .meta { display: flex; justify-content: space-between; color: #666; font-size: 0.9em; margin-bottom: 10px; }
        .score { font-weight: bold; color: #2ecc71; }
        .filename { color: #3498db; font-weight: bold; }
        .calls { font-size: 0.85em; color: #d35400; margin-top: 10px; border-top: 1px solid #eee; padding-top: 5px; }
        .call-tag { background: #fae5d3; padding: 2px 6px; border-radius: 4px; margin-right: 5px; display: inline-block; }
        pre { background: #f8f8f8; padding: 15px; border-radius: 4px; overflow-x: auto; font-size: 0.9em; border: 1px solid #eee; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Search Results</h1>
        <p>Query: <strong>{{ query }}</strong></p>
    </div>

    {% for result in results %}
    <div class="result-card">
        <div class="meta">
            <span class="rank">#{{ result.rank }}</span>
            <span class="filename">{{ result.filename }}:{{ result.line_start }}-{{ result.line_end }}</span>
            <span class="score">Score: {{ "%.4f"|format(result.score) }}</span>
        </div>
        <pre><code>{{ result.code }}</code></pre>
        {% if result.calls %}
        <div class="calls">
            <strong>Calls:</strong> 
            {% for call in result.calls %}
            <span class="call-tag">{{ call }}</span>
            {% endfor %}
        </div>
        {% endif %}
    </div>
    {% endfor %}
</body>
</html>
    "#;

    env.add_template("report", TEMPLATE).unwrap();
    let template = env.get_template("report").unwrap();
    
    template.render(context! {
        query => query,
        results => results,
    }).unwrap_or_else(|e| format!("Error generating report: {}", e))
}
