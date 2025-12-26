# dry-handlebars

_Experimental_ compile-time checked [Handlebars](https://handlebarsjs.com/) templates for Rust.
Based on the parser from [rusty-handlebars](https://github.com/h-i-v-e/rusty-handlebars).

Code first or schema first?
https://blog.logrocket.com/code-first-vs-schema-first-development-graphql/

Designer gets code highlighting and autocomplete of HTML and CSS

Rust developer gets compile time checking of template

SQL libraries like Diesel and SeaORM use code first approach. they imply that SQL is not 
the boss. But they are wrong. SQL should always be the boss.

harness to use a thing

The code is the boss. Whatever the code wants, the SQL must provide. The author should not need to

Why? The template is the boss. Whatever the template wants, the code must provide. The author should not need to 
repeat themselves.

The developer provides the data. The designer can choose to use it or not.

developer provides: name: str, age: u32, role: RoleStruct


This ensures that any errors in the template are found at compile time.
And any errors in passing data to the template are found at compile time.

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

Still in alpha stage, only a subset of handlebars functionality is supported.

- [x] Uses `Display` trait for variables
- [x] Get a struct and a template function for a `str`
- [x] For a single file
- [x] A directory of templates
- [x] Simple top level keys (e.g. `{{ todo_id }}`) -> Fields must implement the `Display` trait
- [x] Item properties (e.g. `{{ person.name }}`) -> Person type alias needed, fields must implement the `Display` trait
- [x] if helpers (e.g. `{{#if ...}} ... {{/if}}`) -> Fields must be `Option<T>`
- [x] if/else helpers (e.g. `{{#if ...}} xxx {{ else }} yyy {{/if}}`) -> Fields must be `Option<T>`
- [ ] For loops (e.g. `{{#each items}} ... {{/each}}`) -> Fields must be iterable via the trait `IntoIterator`

