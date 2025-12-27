# dry-handlebars

_Experimental_ compile-time checked [Handlebars](https://handlebarsjs.com/) templates for Rust.
Based on the parser from [rusty-handlebars](https://github.com/h-i-v-e/rusty-handlebars).

The blog post [code first or schema first](https://blog.logrocket.com/code-first-vs-schema-first-development-graphql/)
highlights that there are two way of thinking about templating. Code first or template first.

This library takes a template first approach. The designs gets pure handlebars files (hbs) that can be edited separately.

The Rust developer gets pure rust experience with compile time checking of templates and how they are called from Rust.

The Rust developer should not have to repeat the template structure in Rust code, hence the name DRY (don't repeat yourself).

Take a directory of handlebars files, for example:

`templates/button.hbs`:

```handlebars

<button id="btn{{ btn_id }}" class="btn btn-light">
    {{ btn_name }}
</button>
```

Usage in rust:

```rust
mod templates {
    dry_handlebars::directory!("templates/");
}
fn get_html() -> String {
    templates::button(42, "Save").render()
}
```

## Features

Still in alpha stage, only a subset of handlebars functionality is supported. Specifically:

- Uses `Display` trait for variables
- Get a struct and a template function for a `str`
- Macro for a directory of templates, single file or a string
- Simple top level keys (e.g. `{{ todo_id }}`) -> Fields must implement the `Display` trait
- Item properties (e.g. `{{ person.name }}`) -> Person type alias needed, fields must implement the `Display` trait
- If helpers (e.g. `{{#if ...}} ... {{/if}}`) -> Fields must be `Option<T>`
- If/else helpers (e.g. `{{#if ...}} xxx {{ else }} yyy {{/if}}`) -> Fields must be `Option<T>`
- For loops (e.g. `{{#each items}} ... {{/each}}`) -> Fields must be iterable via the trait `IntoIterator`
