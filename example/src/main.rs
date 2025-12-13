use dry_handlebars::dry_handlebars_directory;

dry_handlebars_directory!("templates/");


fn main() {
    let html = TodoEdit::new(42, "My Todo").render();
    println!("{}", html);

    struct StructExample {title: String}
    let struct_example = StructExample { title: "example".to_owned() };
    //let h2 = StructMembers::new(struct_example).render();


}
