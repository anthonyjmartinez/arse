# A Rust Site Engine

A Rust Site Engine, or `arse`, is static site generator written in Rust. It seeks to provide a simple
and flexible base for serving sites using:

* [Tera](https://tera.netlify.app/) for templates
* [pulldown-cmark](https://crates.io/crates/pulldown-cmark) for CommonMark rendering
* [routerify](https://crates.io/crates/routerify) to serve the site
* [simplecss](https://simplecss.org) for default styling

## Usage

* Run an existing site given the path to its config TOML: `arse run config.toml`
* Logging verbosity can be increased with `-v` or `-vv`, the default level is `INFO`.
* Create and run a new site from user input: `arse new`

```
$ arse new
2021-05-01T17:34:11.155427589+00:00 [INFO] Logging started
2021-05-01T17:34:11.155543182+00:00 [INFO] Generating new site configuration
Please enter a name for the site: 
Example Site
Please enter the site author's name: 
Arthur Writeson
Please enter comma-separated site topics: 
one, two, three
2021-05-01T17:34:26.501980660+00:00 [INFO] Creating site filesystem tree
2021-05-01T17:34:26.504816188+00:00 [INFO] Writing site configuration to disk
```

### Configuration

An example config, as generated, is shown below:

```toml
[site]
name = 'Example Site'
author = 'Arthur Writeson'
template = 'default.tmpl'
topics = [
    'one',
    'two',
    'three',
]

[server]
bind = '0.0.0.0'
port = 9090

[docpaths]
templates = '/home/user/site/templates'
webroot = '/home/user/site/webroot'
```

#### Rendering and Styling

A default template, `default.tmpl`, is provided statically within the binary. To change the Tera
template, add your custom template to the templates directory referenced in the `[docpaths]` configuration
section of `config.toml`. Once the template is in the templates directory, change the `templates` parameter
in the `[site]` configuration section to reference the template's file name. This template will now be loaded
at runtime.

The following elements are available within the Tera context for rendering:

* `site`, mapping directly to the fields available in the `site` configuration section
* `post`, available when serving single-posts from from `site/:topic/posts/:post.md`
  * Used when serving `GET /:topic/posts/:post` where `:post` is the markdown filename minus its extension
* `posts`, a lexically reverse-sorted list of HTML rendered from markdown in `site/:topic/posts/*.md`
  * Used when serving `GET /:topic`

#### Further Customizations

* `bind` and `port` may be set in the `[server]` section.
* New topics are added as array elements
  * For each new topic, create the necessary paths `site/:topic/posts` and `site/:topic/ext`
* Items in `[docpaths]` are generated as full paths for completeness, however relative paths will work if desired
  * From the example above the user is free to simply use `site/templates` and `site/webroot` and move the directory out of `/home/user`
  * Note that `arse new` creates the site tree, and all other output files, in the current working directory.

## Path to 1.0

- [x] Dynamic route handling
- [x] Provide meaningful logging of binary activites at appropriate levels
- [x] Context-specific Errors and handling
- [x] Support custom Tera templates
- [x] Support custom bind address and port
- [x] Support favicons
- [ ] Support RSS feeds

### License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
