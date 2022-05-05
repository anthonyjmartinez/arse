/*
A Rust Site Engine
Copyright 2020-2021 Anthony Martinez

Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
http://opensource.org/licenses/MIT>, at your option. This file may not be
copied, modified, or distributed except according to those terms.
*/

/// Default [`tera`] template for A Rust Site Engine's rendering engine
pub(crate) const TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<link rel="stylesheet" href="https://cdn.simplecss.org/simple.min.css">
<title>{{ site.name }}</title>
</head>
<body>
<header>
<h1>{{ site.name }}</h1>
<nav>
<a href="/">Home</a>
{%- for topic in site.topics %}
<a href="/{{ topic | slugify }}">{{ topic }}</a>
{%- endfor -%}
<a href="/rss.xml">RSS</a>
</nav>
</header>
<main>
{% if gallery %}
<script>
var images = new Array();
{%- for img in gallery %}
images[{{ loop.index0 }}] = "{{ img }}";
{%- endfor -%}
var imgCt = 0;
function change_img(dir) {
        if (dir == "next" && imgCt < images.length - 1) {
                imgCt++;
        }

        if (dir == "prev" && imgCt > 0) {
                imgCt--;
        } 

        var doc = document.getElementById("gallery");
        var img = new Image();
        img.src = images[imgCt];

        doc.replaceChildren(img);
}
</script>
<center>
<div id="gallery">
<img src="{{ gallery | first }}"/>
</div>
<button type="button" onclick="change_img('prev'); return false">❮</button>
<button type="button" onclick="change_img('next'); return false">❯</button>
</center>
{% elif post %}
{{ post }}
{% elif posts %}
{%- for post in posts %}
{{ post }}
{%- endfor -%}
{% else %}
<h3>Coming Soon!</h3>
{% endif %}
</main>
<footer>
<p>&#169; {{ site.author }}</p>
</footer>
</body>
</html>
"#;
