# dry-handlebars

Compile-time checked [Handlebars](https://handlebarsjs.com/) templates for Rust.

The template is the boss. Whatever the template wants, the code must provide.

Based on the parser from [rusty-handlebars](https://github.com/h-i-v-e/rusty-handlebars).

Take a directory of handlebars files, for example:

`templates/TodoEdit.hbs`:
```handlebars
<button id="todo-edit-{{ todo_id }}" class="btn btn-light">
    {{ todo_name }}
</button>
```

And have rust code that looks like this:
```rust
mod templates {
    dry_handlebars::directory!("templates/");
}
fn get_html() -> String {
    templates::todo_edit(42, "My Todo").render()
}
```

This ensures that any errors in the template are found at compile time.
And any errors in passing data to the template are found at compile time.

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

