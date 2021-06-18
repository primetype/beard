/*!
# beard

In opposition to [mustache]. Here the goal instead of mustache is
to leverage as much as possible rust's type system to detect error
case and therefor to make the rendering deterministic. If you are
looking for something that is going to be portable outside of rust
you should checkout [mustache].

[`beard`] is a macro that will generate the necessary rust code
to serialise the given _template_. You can achieve the same thing
by writing the code yourself (calling [std::io::Write] appropriate
methods). [`beard`] is simply an help to do that and to make it
easier to maintain the templates.

# How it works

## string literals

Any string literal will be written as is in the output.

```
use beard::beard;
# use std::io::Write as _;
#
# fn render() -> Result<String, std::io::Error> {
#    let mut output = Vec::new();
    beard! {
        output,
        "List of items:\n"
        "\n"
        "* item 1\n"
        "* item 2\n"
    };
#    Ok(String::from_utf8(output).unwrap())
# }
# let output = render().unwrap();
```

## serialising any impl of Display

You can intersperse serialised object that implements [`std::fmt::Display`],
just use the curly bracket as you would if you were to use in
[`std::format`] macro, except outside of the string.

Interestingly it can also do any kind of operations within the brackets
so long it returns something that implements [`std::fmt::Display`].

```
use beard::beard;
# use std::io::Write as _;
#
# fn render() -> Result<String, std::io::Error> {
#    let mut output = Vec::new();
    let value = 42;
    beard! {
        output,
        { value } "\n"
        { 1 + 2 } "\n"
    };
#    Ok(String::from_utf8(output).unwrap())
# }
# let output = render().unwrap();
```

## serialising array of bytes

In case the value is already an array and there is no need to run
[`std::fmt::Display`]'s formatting to [`String`] as intermediate.

```
use beard::beard;
# use std::io::Write as _;
#
# fn render() -> Result<String, std::io::Error> {
#    let mut output = Vec::new();
    let value = b"some array";
    beard! {
        output,
        [{ value }] "\n"
    };
#    Ok(String::from_utf8(output).unwrap())
# }
# let output = render().unwrap();
```

## Calling function to serialize

Would you need to perform some operation with the [`std::io::Write`]
object you can capture the variable by calling a _lambda_ (it's not
really one).

```
use beard::beard;
# use std::io::Write as _;
#
# fn render() -> Result<String, std::io::Error> {
#    let mut output = Vec::new();
    let value = b"some array";
    beard! {
        output,
        || {
            // do other operations
            output.write_all(b"some bytes")?;
        }
    };
#    Ok(String::from_utf8(output).unwrap())
# }
# let output = render().unwrap();
```

## `if` and `if let` statement

There are times where one needs to serialize or not some parts.

```
use beard::beard;
# use std::io::Write as _;
#
# fn render() -> Result<String, std::io::Error> {
#    let mut output = Vec::new();
    let value = false;
    let optional = Some("something");
    beard! {
        output,
        if (value) {
            "Value is true\n"
        } else {
            "Value is false\n"
        }

        "Or do some pattern matching\n"
        if let Some(value) = (optional) {
            "We have " { value } "\n"
        }
    };
#    Ok(String::from_utf8(output).unwrap())
# }
# let output = render().unwrap();
```

## `for` loop, iterating on items

Shall you need to print the items of a list or anything that
implements [`std::iter::IntoIterator`].

```
use beard::beard;
# use std::io::Write as _;
#
# fn render() -> Result<String, std::io::Error> {
#    let mut output = Vec::new();
    let shopping_list = ["apple", "pasta", "tomatoes", "garlic", "mozzarella"];
    beard! {
        output,
        "Shopping list\n"
        "\n"
        for (index, item) in (shopping_list.iter().enumerate()) {
            {index} ". " { item } "\n"
        }
    };
#    Ok(String::from_utf8(output).unwrap())
# }
# let output = render().unwrap();
```

# Example

```
use beard::beard;
# use std::io::Write as _;
#
# fn render() -> Result<String, std::io::Error> {
    let name = "Arthur";
    let list = ["Bubble Bath", "Unicorn Crunchy Oat"];

#    let mut output = Vec::new();
    beard! {
        output,
        "Hi " { name } "\n"
        "\n"
        "Confirmation order about the following items:\n"
        for item in ( list ) {
            " * " { item } "\n"
        }
        "\n"
        "Your order will be ship to you once everything is ready.\n"
    };
#    Ok(String::from_utf8(output).unwrap())
# }
# let output = render().unwrap();
```

The Example below will generate a string in the `output`:

```text
Hi Arthur

Confirmation order about the following items:
 * Bubble Bath
 * Unicorn Crunch Oat

Your order will be ship to you once everything is ready.
```

[`beard`]: ./macro.beard.html
[mustache]: https://mustache.github.io/mustache.5.html
*/

/// macro to call to generate the function stream of generating
/// formatted output.
///
/// The difference here with [`std::fmt::format`] is that instead
/// generating a string based on some formatting parameters
/// the [`beard`] macro generates a string based on the declarative
/// flow.
#[macro_export]
macro_rules! beard {
    ($($any:tt)*) => {
        $crate::beard_internal!($($any)*);
    };
}

/// use this internal macro to hide the details of the macro away
///
/// this is not really useful for the user documentation anyway.
#[macro_export]
#[doc(hidden)]
macro_rules! beard_internal {
    ($output:ident, ) => {
    };

    ($output:ident, | | $statement:block $($any:tt)*) => {
        {
            $statement
        }
        $crate::beard_internal!($output, $($any)*);
    };
    ($output:ident, || $statement:block $($any:tt)*) => {
        {
            $statement
        }
        $crate::beard_internal!($output, $($any)*);
    };


    ($output:ident, $text:literal $($any:tt)*) => {
        $output.write_all($text.as_bytes())?;
        $crate::beard_internal!($output, $($any)*);
    };
    ($output:ident, [ $statement:block ] $($any:tt)*) => {
        $output.write_all(
             $statement.as_ref()
        )?;
        $crate::beard_internal!($output, $($any)*);
    };
    ($output:ident, $statement:block $($any:tt)*) => {
        $output.write_all(
             $statement.to_string().as_bytes()
        )?;
        $crate::beard_internal!($output, $($any)*);
    };

    ($output:ident, if ( $condition:expr ) { $($statement:tt)+ } else { $($alternative:tt)+ } $($any:tt)*) => {
        if $condition {
            $crate::beard_internal!($output, $($statement)+);
        } else {
            $crate::beard_internal!($output, $($alternative)+);
        }
        $crate::beard_internal!($output, $($any)*);
    };
    ($output:ident, if let $condition:pat = ( $value:expr ) { $($statement:tt)+ } $($any:tt)*) => {
        if let $condition = $value {
            $crate::beard_internal!($output, $($statement)+);
        }
        $crate::beard_internal!($output, $($any)*);
    };
    ($output:ident, if ( $condition:expr ) { $($statement:tt)+ } $($any:tt)*) => {
        if $condition {
            $crate::beard_internal!($output, $($statement)+);
        }
        $crate::beard_internal!($output, $($any)*);
    };

    ($output:ident, for $value:pat in ($into_iter:expr) { $($statement:tt)+ } $($any:tt)*) => {
        for $value in $into_iter.into_iter() {
            #![allow(clippy::into_iter_on_ref, array_into_iter)]
            $crate::beard_internal!($output, $($statement)+);
        }
        $crate::beard_internal!($output, $($any)*);
    };
}

#[test]
fn test() {
    use std::io::Write as _;

    const EXPECTED: &str = r##"Variables can be formatted as follow: value.
Statement works too: 3 (so you can do special formatting if you want).
 as bytes directly: value
The length of the stuff is not null value
Optional value set 1
Optional value not set
print thing: one
print thing: two
something custom"##;

    fn render() -> Result<String, std::io::Error> {
        let value = "value";
        let stuff = ["one", "two"];
        let optionals = [Some(1), None];

        let mut output = Vec::new();
        beard! {
            output,
            "Variables can be formatted as follow: " { value } ".\n"
            "Statement works too: " { 1 + 2} " (so you can do special formatting if you want).\n"
            if (value == "something") {
                "This test is not rendered" { value }
            }

            " as bytes directly: " [ { value.as_bytes() } ] "\n"

            if (!stuff.is_empty()) {
                "The length of the stuff is not null " { value } "\n"
            } else {
                "oops\n"
            }


            for optional in ( optionals ) {
                if let Some(value) = ( optional ) {
                    "Optional value set " { value } "\n"
                }
                if let None = (optional) {
                    "Optional value not set\n"
                }
            }

            for (_index, thing) in (stuff.iter().enumerate()) {

                "print thing: " { thing } "\n"
            }

            | | { output.write_all(b"something custom")?; }
        };
        Ok(String::from_utf8(output).unwrap())
    }

    let message = render().unwrap();

    println!("{}", message);

    assert_eq!(EXPECTED, message);
}
