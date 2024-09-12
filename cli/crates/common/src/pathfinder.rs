pub fn index_html(graphql_url: &str, asset_url: &str) -> String {
    format!(
        r#"
    <html lang="en">
      <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Pathfinder – Grafbase</title>
        <link rel="shortcut icon" href="{asset_url}/images/grafbase-logo-circle.png" />
        <link rel="stylesheet" href="{asset_url}/assets/index.css" />
        <script>
          window.GRAPHQL_URL = '{graphql_url}';
        </script>
        <script type="module" crossorigin src="{asset_url}/assets/index.js"></script>
      </head>
      <body>
        <div id="root"></div>
      </body>
    </html>
    "#
    )
}
