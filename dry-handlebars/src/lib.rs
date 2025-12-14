
pub use dry_handlebars_macros::dry_handlebars_directory as directory;
pub use dry_handlebars_macros::dry_handlebars_file as file;
pub use dry_handlebars_macros::dry_handlebars_str as str;


#[cfg(test)]
mod tests {

    #[test]
    fn basic_usage() {
        mod template {
            crate::str!("test", r#"<p>{{firstname}} {{lastname}}</p>"#);
        }
        assert_eq!(
            template::test(" King", "Tubby ").render(),
            "<p> King Tubby </p>"
        );
    }

    struct Person {
        firstname: String,
        lastname: String,
    }

    #[test]
    fn path_expressions() {
        mod template {
            crate::str!(
                "test",
                r#"{{person.firstname}} {{person.lastname}}"#,
                ("person", super::Person)
            );
        }
        let person = Person {
            firstname: "King".to_string(),
            lastname: "Tubby".to_string(),
        };
        assert_eq!(template::test(person).render(), "King Tubby");
    }

    struct Author {
        first_name: String,
        last_name: String,
    }

    #[test]
    fn if_helper() {
        mod template {
            crate::str!(
                "test",
                //language=html
                r#"
                <div class="entry">
                    {{#if author}}
                        <h1>{{first_name}} {{last_name}}</h1>
                    {{/if}}
                </div>
            "#,
                ("author", super::Author)
            );
        }
        let author = Author {
            first_name: "King".to_string(),
            last_name: "Tubby".to_string(),
        };
        assert_eq!(
            template::test(Some(author)).render().trim(),
            //language=html
            r#"<div class="entry">

                        <h1>King Tubby</h1>

                </div>"#
        );
        assert_eq!(
            template::test(None).render().trim(),
            //language=html
            r#"<div class="entry">

                </div>"#
        );
    }

    #[test]
    fn if_else_helper() {
        mod template {
            crate::str!(
                "test",
                //language=html
                r#"
                <div class="entry">
                    {{#if author}}<h1>{{first_name}}</h1>{{else}}<h1>Unknown</h1>{{/if}}
                </div>
            "#,
                ("author", super::Author)
            );
        }
        let author = Author {
            first_name: "King".to_string(),
            last_name: "Tubby".to_string(),
        };
        assert_eq!(
            template::test(Some(author)).render().trim(),
            //language=html
            r#"<div class="entry">
                    <h1>King</h1>
                </div>"#
        );
        assert_eq!(
            template::test(None).render().trim(),
            //language=html
            r#"<div class="entry">
                    <h1>Unknown</h1>
                </div>"#
        );
    }
}
