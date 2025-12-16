
mod templates {
    dry_handlebars::directory!("templates/");
    dry_handlebars::file!("template/button2.hbs");
    //language=html
    dry_handlebars::str!("hello_first_last", r#"
        <p>Hello {{firstname}} {{lastname}}</p>
    "#);
}

fn main() {
    let html = templates::button(42, "My Todo").render();
    println!("{}", html);

    let html2 = templates::button2(43, "Single File Todo").render();
    println!("{}", html2);

    let html3 = templates::hello_first_last("King", "Tubby").render();
    println!("{}", html3);
}
