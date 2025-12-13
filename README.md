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

Still in alpha stage, but features planned:

- [x] Uses `Display` trait for variables
- [x] Get a struct and a template function for a `str`
- [x] For a single file
- [x] A directory of templates
- [x] Support for simple top level keys (e.g. `{{ todo_id }}`)
- [ ] Support for block helpers (e.g. `{{#if ...}} ... {{/if}}`)
- [ ] Support for for loops (e.g. `{{#each items}} ... {{/each}}`)
- [ ] Support for nested keys (e.g. `{{ user.name }}`)
