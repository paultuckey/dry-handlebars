mod test;

mod templates {
    dry_handlebars::directory!("templates/");
    dry_handlebars::file!("template/TodoEdit2.hbs");
    dry_handlebars::str!("basic_usage", r#"
        <p>{{firstname}} {{lastname}}</p>
    "#);
}

fn main() {
    let html = templates::todo_edit(42, "My Todo").render();
    println!("{}", html);

    let html2 = templates::todo_edit2(43, "Single File Todo").render();
    println!("{}", html2);

    let html3 = templates::basic_usage("King", "Tubby").render();
    println!("{}", html3);
}
