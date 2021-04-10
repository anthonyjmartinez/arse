// A Rust Site Engine
// Copyright 2020-2021 Anthony Martinez
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

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
