pub const TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<link rel="stylesheet" href="https://cdn.simplecss.org/simple.min.css">
<title>{{ blog.name }}</title>
</head>
<body>
<header>
<center>
<h1>{{ blog.name }}</h1>
<nav>
<a href="./">Home</a>
{%- for topic in blog.topics %}
<a href="./{{ topic | slugify }}">{{ topic }}</a>
{%- endfor -%}
</nav>
</center>
</header>
<main>
{% if posts | length == 0 %}
<h3>Coming Soon!</h3>
{% else %}
{%- for post in posts %}
{{ post }}
{%- endfor -%}
{% endif %}
</main>
<footer>
<p>&#169; {{ blog.author }}</p>
</footer>
</body>
</html>
"#;
