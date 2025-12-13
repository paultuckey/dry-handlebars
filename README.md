# dry-handlebars

Why oh why another templating library.

Compile time, very strict.

The template is god. Whatever the template wants, the code must provide.

Building on the amazing [handlebars](https://github.com/sunng87/handlebars-rust) crate for rust.

Take a directory (`templates/`) of handlebars files (`TodoEdit.hbs`) like this:

```handlebars
<button id="todo-edit-{{ todo_id }}" class="btn btn-light">
    {{ todo_name }}
</button>
```

And have rust code that looks like this:

```rust
dry_handlebars_directory!("templates/");

fn get_html() -> String {
    templates::TodoEdit::new(42, "My Todo").render()
}
```

##

```mustache
{{hello}} {{world}}

```
```smarty
{{hello}} {{world}}
```

