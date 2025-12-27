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
            template::test("King", "Tubby").render(),
            "<p>King Tubby</p>"
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
                //language=handlebars
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
                //language=handlebars
                r#"<div>{{#if has_author}}<h1>{{first_name}} {{last_name}}</h1>{{/if}}</div>"#
            );
        }
        assert_eq!(
            template::test(true, "King", "Tubby").render(),
            //language=html
            "<div><h1>King Tubby</h1></div>"
        );
        assert_eq!(
            template::test(false, "King", "Tubby").render(),
            //language=html
            "<div></div>"
        );
    }

    #[test]
    fn unless_helper() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#unless has_author}}<h1>Unknown</h1>{{/unless}}</div>"#
            );
        }
        assert_eq!(
            template::test(false).render(),
            //language=html
            "<div><h1>Unknown</h1></div>"
        );
        assert_eq!(
            template::test(true).render(),
            //language=html
            "<div></div>"
        );
    }

    #[test]
    fn if_else_helper() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#if has_author}}<h1>{{first_name}}</h1>{{else}}<h1>Unknown</h1>{{/if}}</div>"#,
                ("author", Option<super::Author>)
            );
        }
        assert_eq!(
            template::test(true, "King").render(),
            //language=html
            r#"<div><h1>King</h1></div>"#
        );
        assert_eq!(
            template::test(false, "King").render(),
            //language=html
            r#"<div><h1>Unknown</h1></div>"#
        );
    }

    #[test]
    fn with_helper_option() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#with author}}<h1>{{first_name}} {{last_name}}</h1>{{/with}}</div>"#,
                ("author", Option<super::Author>)
            );
        }
        let author = Author {
            first_name: "King".to_string(),
            last_name: "Tubby".to_string(),
        };
        assert_eq!(
            template::test(Some(author)).render(),
            //language=html
            "<div><h1>King Tubby</h1></div>"
        );
        assert_eq!(
            template::test(None).render(),
            //language=html
            "<div></div>"
        );
    }

    #[test]
    fn with_helper() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#with author}}<h1>{{first_name}} {{last_name}}</h1>{{/with}}</div>"#,
                ("author", super::Author)
            );
        }
        let author = Author {
            first_name: "King".to_string(),
            last_name: "Tubby".to_string(),
        };
        assert_eq!(
            template::test(author).render(),
            //language=html
            "<div><h1>King Tubby</h1></div>"
        );
    }

    #[test]
    fn for_helper() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"<div>{{#each authors}}<p>Hello {{first_name}}</p>{{/each}}</div>"#,
                ("authors", Vec<super::Author>)
            );
        }
        let author = Author {
            first_name: "King".to_string(),
            last_name: "Tubby".to_string(),
        };
        assert_eq!(
            template::test(vec![author]).render(),
            //language=html
            "<div><p>Hello King</p></div>"
        );
    }

    #[test]
    fn test_comment() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"Note: {{! This is a comment }} and {{!-- {{so is this}} --}}\\{{{{}}"#,
            );
        }
        assert_eq!(template::test().render(), "Note:  and \\{{");
    }

    #[test]
    fn test_trimming() {
        mod template {
            crate::str!(
                "test",
                //language=handlebars
                r#"  {{~#if some ~}}   Hello{{~/if~}}"#,
            );
        }
        assert_eq!(template::test(true).render(), "Hello");
    }

    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///

    #[test]
    fn it_works() {
        mod template {
            crate::str!("test", "Hello {{{name}}}!");
        }
        assert_eq!(template::test("King").render(), "Hello King!");
    }

    #[test]
    fn test_escaped() {
        mod template {
            crate::str!(
                "test",
                "{{{{skip}}}}wang doodle {{{{/dandy}}}}{{{{/skip}}}}"
            );
        }
        assert_eq!(template::test().render(), "wang doodle {{{{/dandy}}}}");
    }

    #[test]
    fn test_format_number() {
        mod template {
            crate::str!("test", "Price: ${{format \"{:.2}\" price}}");
        }
        assert_eq!(template::test(12.2345f64).render(), "Price: $12.23");
    }

    // #[test]
    // fn test_nesting() {
    //     let rust = compile("{{#if some}}{{#each some}}Hello {{this}}{{/each}}{{/if}}");
    //     assert_eq!(
    //         rust,
    //         "if self.some.as_bool(){for this_2 in self.some{write!(f, \"Hello {}\", this_2.as_display_html())?;}}"
    //     );
    // }
    //
    // #[test]
    // fn test_as() {
    //     let rust = compile(
    //         "{{#if some}}{{#each some as thing}}Hello {{thing}} {{thing.name}}{{/each}}{{/if}}",
    //     );
    //     assert_eq!(
    //         rust,
    //         "if self.some.as_bool(){for thing_2 in self.some{write!(f, \"Hello {} {}\", thing_2.as_display_html(), thing_2.name.as_display_html())?;}}"
    //     );
    // }
    //
    // #[test]
    // fn test_scoping() {
    //     let rust = compile(
    //         "{{#with some}}{{#with other}}Hello {{name}} {{../company}} {{/with}}{{/with}}",
    //     );
    //     assert_eq!(
    //         rust,
    //         "{let this_1 = self.some;{let this_2 = this_1.other;write!(f, \"Hello {} {} \", this_2.name.as_display_html(), this_1.company.as_display_html())?;}}"
    //     );
    // }
    //
    // #[test]
    // fn test_indexer() {
    //     let rust = compile(
    //         "{{#each things}}Hello{{{@index}}}{{#each things}}{{{lookup other @../index}}}{{{@index}}}{{/each}}{{/each}}",
    //     );
    //     assert_eq!(
    //         rust,
    //         "let mut i_1 = 0;for this_1 in self.things{write!(f, \"Hello{}\", i_1.as_display())?;let mut i_2 = 0;for this_2 in this_1.things{write!(f, \"{}{}\", this_2.other[i_1].as_display(), i_2.as_display())?;i_2+=1;}i_1+=1;}"
    //     );
    // }
    //
    // #[test]
    // fn test_map() {
    //     let rust = compile(
    //         "{{#each things}}Hello{{{@key}}}{{#each @value}}{{#if_some (try_lookup other @../key)}}{{{this}}}{{/if_some}}{{{@value}}}{{/each}}{{/each}}",
    //     );
    //     assert_eq!(
    //         rust,
    //         "for this_1 in self.things{write!(f, \"Hello{}\", this_1.0.as_display())?;for this_2 in this_1.1{if let Some(this_3) = this_2.other.get(this_1.0){write!(f, \"{}\", this_3.as_display())?;}write!(f, \"{}\", this_2.1.as_display())?;}}"
    //     );
    // }
    //
    //
    // #[test]
    // fn test_subexpression() {
    //     let rust = compile(
    //         "{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}",
    //     );
    //     assert_eq!(
    //         rust,
    //         "let mut i_1 = 0;for this_1 in self.things{{let other_2 = self.other[i_1];write!(f, \"{}: {}\", this_1.name.as_display(), other_2.as_display())?;}i_1+=1;}"
    //     );
    // }
    //
    // #[test]
    // fn test_selfless() {
    //     let rust = Compiler::new(Options{
    //         root_var_name: None,
    //         write_var_name: "f",
    //         variable_types: Default::default(),
    //     }, make_map()).compile("{{#each things}}{{#with (lookup ../other @index) as |other|}}{{{../name}}}: {{{other}}}{{/with}}{{/each}}").unwrap();
    //     assert_eq!(
    //         rust.uses("rusty_handlebars").to_string(),
    //         "use rusty_handlebars::AsDisplay"
    //     );
    //     assert_eq!(
    //         rust.code,
    //         "let mut i_1 = 0;for this_1 in things{{let other_2 = other[i_1];write!(f, \"{}: {}\", this_1.name.as_display(), other_2.as_display())?;}i_1+=1;}"
    //     );
    // }
    //
    // #[test]
    // fn javascript() {
    //     let rust = Compiler::new(opts(), make_map()).compile("<script>if (location.href.contains(\"localhost\")){ console.log(\"\\{{{{}}}}\") }</script>").unwrap();
    //     assert_eq!(rust.uses("rusty_handlebars").to_string(), "");
    //     assert_eq!(
    //         rust.code,
    //         "write!(f, \"<script>if (location.href.contains(\\\"localhost\\\")){{ console.log(\\\"{{{{}}}}\\\") }}</script>\")?;"
    //     );
    // }
}
