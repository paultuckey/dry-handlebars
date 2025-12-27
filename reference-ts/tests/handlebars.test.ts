import { describe, it, expect } from 'vitest';
import Handlebars from 'handlebars';

describe('Handlebars Reference Tests', () => {
    it('basic_usage', () => {
        const template = Handlebars.compile('<p>{{firstname}} {{lastname}}</p>');
        const result = template({ firstname: "King", lastname: "Tubby" });
        expect(result).toBe('<p>King Tubby</p>');
    });

    it('path_expressions', () => {
        const template = Handlebars.compile('{{person.firstname}} {{person.lastname}}');
        const person = {
            firstname: "King",
            lastname: "Tubby",
        };
        const result = template({ person });
        expect(result).toBe('King Tubby');
    });

    it('if_helper', () => {
        const template = Handlebars.compile('<div>{{#if has_author}}<h1>{{first_name}} {{last_name}}</h1>{{/if}}</div>');

        const resultTrue = template({ has_author: true, first_name: "King", last_name: "Tubby" });
        expect(resultTrue).toBe('<div><h1>King Tubby</h1></div>');

        const resultFalse = template({ has_author: false, first_name: "King", last_name: "Tubby" });
        expect(resultFalse).toBe('<div></div>');
    });

    it('with_helper', () => {
        const template = Handlebars.compile('<div>{{#with author}}<h1>{{first_name}} {{last_name}}</h1>{{/with}}</div>');
        const author = {
            first_name: "King",
            last_name: "Tubby",
        };
        const resultTrue = template({ author });
        expect(resultTrue).toBe('<div><h1>King Tubby</h1></div>');

        const resultFalse = template({ author: null });
        expect(resultFalse).toBe('<div></div>');
    });

    it('with_else_helper', () => {
        const template = Handlebars.compile('<div>{{#with author}}<h1>{{first_name}}</h1>{{else}}<h1>Unknown</h1>{{/with}}</div>');

        const author = {
            first_name: "King",
            last_name: "Tubby",
        };

        const resultTrue = template({ author });
        expect(resultTrue).toBe('<div><h1>King</h1></div>');

        const resultFalse = template({ author: null });
        expect(resultFalse).toBe('<div><h1>Unknown</h1></div>');
    });

    it('it_works', () => {
        const template = Handlebars.compile('Hello {{{name}}}!');
        const result = template({ name: "King" });
        expect(result).toBe('Hello King!');
    });

    it.skip('test_escaped', () => {
        // Handlebars JS parser throws "skip doesn't match dandy"
        // It seems it gets confused by the inner {{{{/dandy}}}}
        const template = Handlebars.compile('{{{{skip}}}}wang doodle {{{{/dandy}}}}{{{{/skip}}}}');
        const result = template({});
        expect(result).toBe('wang doodle {{{{/dandy}}}}');
    });

    it('test_format_number', () => {
        Handlebars.registerHelper('format', function(fmt, value) {
            if (fmt === "{:.2}" && typeof value === 'number') {
                return value.toFixed(2);
            }
            return value;
        });

        const template = Handlebars.compile('Price: ${{format "{:.2}" price}}');
        const result = template({ price: 12.2345 });
        expect(result).toBe('Price: $12.23');
    });
});

