#[macro_use]
extern crate nom;
extern crate clap;

use nom::*;
use clap::{Arg, App};

use std::str::Chars;
use std::ascii::AsciiExt;


const TAPE_SIZE: usize = 32000;
type Tape = [u8; TAPE_SIZE];

fn main() {
    // read arguments
    let matches = App::new("Rust Brainfuck")
        .version("0.1.0")
        .author("Henry Zimmerman")
        .about("A Brainfuck interpreter written in Rust.")
        .arg(
            Arg::with_name("input")
            .short("i")
            .long("input")
            .value_name("INPUT")
            .help("A string for your Brainfuck program to read. (',' character).")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("brainfuck")
            .short("b")
            .long("brainfuck")
            .value_name("CODE")
            .help("The Brainfuck program you wish to execute.")
            .takes_value(true)
            .required(true)
        )
        .get_matches();
    let input_str: String = matches.value_of("input").unwrap_or("").to_string();
    let mut input_chars = input_str.chars();
    let bf_input: String = matches.value_of("brainfuck").unwrap_or("").to_string();


    // Set up tape
    let mut tape: Tape = [0; TAPE_SIZE];
    let mut tape_pointer: usize = 0;

    // parse the tokens, run BF on tokens.
    let tokens = parse_input(bf_input);
    consume_tokens(&tokens, &mut tape, &mut tape_pointer, &mut input_chars);
}


#[derive(Debug, PartialEq, Clone)]
enum Token {
    Plus,
    Minus,
    ShiftRight,
    ShiftLeft,
    Output,
    Input,
    Loop {expr: Vec<Token>},
    Comment
}


fn consume_tokens(tokens: &Vec<Token>, tape: &mut Tape, tape_pointer: &mut usize, input: &mut Chars) -> String {
    let mut output_string: String = String::new();

    for token in tokens {
        match *token {
            Token::Plus => {
                tape[*tape_pointer] += 1;
            },
            Token::Minus => {
                tape[*tape_pointer] -= 1;
            },
            Token::ShiftRight => {
                *tape_pointer += 1;
            },
            Token::ShiftLeft => {
                *tape_pointer -= 1;
            },
            Token::Output => {
                print!("{}", tape[*tape_pointer] as char);
                output_string.push(tape[*tape_pointer] as char);
            },
            Token::Input => {
                match input.next() {
                    Some(c) => {
                        if c.is_ascii() {
                            let value: u8 = c as u8;
                            tape[*tape_pointer] = value; // set the value at the ptr to be the value of the input character.
                        } else {
                            panic!("character {} is not ascii", c);
                        }
                    },
                    None => {
                        panic!("Ran out of input");  // todo, look for some spec on BF to find out what to do here. Should I loop back around the input?
                    }
                };
            },
            Token::Loop { ref expr } => {
                while tape[*tape_pointer] > 0 {
                    consume_tokens(&expr, tape, tape_pointer, input);
                }
            }
            _ => {}
        }
    }

    output_string
}



named!(plus_parser<Token>,
    do_parse!(
        tag!("+") >>
        (Token::Plus)
    )
);

named!(minus_parser<&[u8], Token>,
    do_parse!(
        tag!("-") >>
        (Token::Minus)
    )
);

named!(shiftr_parser<&[u8], Token>,
    do_parse!(
        tag!(">") >>
        (Token::ShiftRight)
    )
);

named!(shiftl_parser<&[u8], Token>,
    do_parse!(
        tag!("<") >>
        (Token::ShiftLeft)
    )
);

named!(output_parser<&[u8], Token>,
    do_parse!(
        tag!(".") >>
        (Token::Output)
    )
);

named!(input_parser<&[u8], Token>,
    do_parse!(
        tag!(",") >>
        (Token::Input)
    )
);

named!(comment_parser<&[u8], Token>,
    do_parse!(
        tag!("//") >>
        complete!(many_till!(anychar, line_ending)) >>
        (Token::Comment)
    )
);

named!(any_end,
    complete!(alt!(line_ending | eof!()))
);

named!(loop_parser<&[u8], Token>,
    do_parse!(
        expression: ws!(delimited!(tag!("["), many0!(syntax), tag!("]")))>>
        (Token::Loop {expr: expression})
    )
);

named!(syntax<Token>,
    alt!( plus_parser | minus_parser | shiftr_parser | shiftl_parser | output_parser | input_parser | loop_parser | comment_parser )
);

named!(brainfuck_parser<&[u8], Vec<Token> >,
    do_parse!(
        tokens: many0!(ws!(syntax)) >>
        (tokens)
    )
);

fn parse_input(input: String) -> Vec<Token> {
    let (_,b) = brainfuck_parser(input.as_bytes()).unwrap();
    b
}




#[test]
fn plus_parser_test() {
    let plus = &b"+"[..];
    let res = plus_parser(plus);
    let remainder = &b""[..];
    assert_eq!(res, IResult::Done(remainder, Token::Plus));
}

#[test]
fn syntax_test() {
    let syn = &b"-"[..];
    let remainder = &b""[..];
    let res = syntax(syn);
    assert_eq!(res, IResult::Done(remainder, Token::Minus));
}

#[test]
fn loop_test() {
    let looop = &b"[++-]"[..];
    let remainder = &b""[..];
    let res = loop_parser(looop);
    use Token::*;
    assert_eq!(res, IResult::Done(remainder, Token::Loop {expr: vec!(Plus, Plus, Minus)}));
}

#[test]
fn nested_loop_test() {
    let looop = &b"[+[++]-]"[..];
    let remainder = &b""[..];
    let res = loop_parser(looop);

    use Token::*;
    assert_eq!(res, IResult::Done(remainder, Token::Loop {expr: vec!(Plus, Loop {expr: vec!(Plus, Plus)}, Minus)}));
}

#[test]
fn ignore_whitespace_test() {
    let bf = &b"+-+>  <  -

    +"[..];
    let remainder = &b""[..];
    let res = brainfuck_parser(bf);

    use Token::*;
    assert_eq!(res, IResult::Done(remainder, vec!(Plus, Minus, Plus, ShiftRight, ShiftLeft, Minus, Plus)));
}


// tests for end of line for the comment
#[test]
fn ignore_comment_eol_test() {
    let bf = &b"+ //+
    +"[..];
    let remainder = &b""[..];
    let res = brainfuck_parser(bf);

    use Token::*;
    assert_eq!(res, IResult::Done(remainder, vec!(Plus, Comment, Plus)));
}

//tests for end of file
//#[test]
fn ignore_comment_eof_test() {
    let bf = &b"+ //"[..];
    let remainder = &b""[..];
    let res = brainfuck_parser(bf);

    use Token::*;
    assert_eq!(res, IResult::Done(remainder, vec!(Plus, Comment)));
}


#[test]
fn hello_world_integration_test() {
    let bf = "++++++++               //Set Cell #0 to 8
[
    >++++               //Add 4 to Cell #1; this will always set Cell #1 to 4
    [                   //as the cell will be cleared by the loop
        >++             //Add 2 to Cell #2
        >+++            //Add 3 to Cell #3
        >+++            //Add 3 to Cell #4
        >+              //Add 1 to Cell #5
        <<<<-           //Decrement the loop counter in Cell #1
    ]                   //Loop till Cell #1 is zero; number of iterations is 4
    >+                  //Add 1 to Cell #2
    >+                  //Add 1 to Cell #3
    >-                  //Subtract 1 from Cell #4
    >>+                 //Add 1 to Cell #6
    [<]                 //Move back to the first zero cell you find; this will
                        //be Cell #1 which was cleared by the previous loop
    <-                  //Decrement the loop Counter in Cell #0
]

>>.                     //Cell #2 has value 72 which is 'H'
>---.                   //Subtract 3 from Cell #3 to get 101 which is 'e'
+++++++..+++.           //Likewise for 'llo' from Cell #3
>>.                     //Cell #5 is 32 for the space
<-.                     //Subtract 1 from Cell #4 for 87 to give a 'W'
<.                      //Cell #3 was set to 'o' from the end of 'Hello'
+++.------.--------.    //Cell #3 for 'rl' and 'd'
>>+.                    //Add 1 to Cell #5 gives us an exclamation point
>++.                    //And finally a newline from Cell #6
".to_string();

    const TAPE_SIZE: usize = 32000;
    let mut tape: Tape = [0; TAPE_SIZE];
    let mut tape_pointer: usize = 0;

    let tokens: Vec<Token> = parse_input(bf);

    let output = consume_tokens(&tokens, &mut tape, &mut tape_pointer, &mut "".chars());
    assert_eq!(output, "Hello World!\n");
}


#[test]
fn multiplication_integration_test() {
    let bf = "+++++++ [>+++<-]>".to_string(); // 7 * 3

    let mut tape: Tape = [0; TAPE_SIZE];
    let mut tape_pointer: usize = 0;

    let tokens: Vec<Token> = parse_input(bf);

    let _ = consume_tokens(&tokens, &mut tape, &mut tape_pointer, &mut "".chars());
    assert_eq!(tape_pointer, 1);
    assert_eq!(tape[tape_pointer], 21);
}

#[test]
fn read_input_test() {
    let bf = ",.>,.".to_string();

    let mut tape: Tape = [0; TAPE_SIZE];
    let mut tape_pointer: usize = 0;

    let tokens: Vec<Token> = parse_input(bf);

    let output = consume_tokens(&tokens, &mut tape, &mut tape_pointer, &mut "HI".chars());
    assert_eq!(tape_pointer, 1);
    assert_eq!(tape[0], 72); // H
    assert_eq!(tape[1], 73); // I
    assert_eq!(output, "HI".to_string());
}