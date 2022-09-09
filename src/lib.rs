mod core; //Module which stores key information such as the type system, tokens, etc.

//Module containing procedures for filehandling, which will be improved over time.
pub mod filehandling {
    use std::fs;
    use std::io::Error as ioError;

    pub fn read_file(file: &str) -> Result<String, ioError> {
        match fs::read_to_string(file) {
            Ok(contents) => Ok(contents),
            Err(e) => Err(e)
        }
    }
}

//Module containing the lexer, a component designed to parse text into tokens that can then be analyzed.
pub mod lexer {
    use crate::core::tokens;
    use std::fs;
        
    //The lexer, a function which converts the text String into tokens, stored in order as a Vector of enum variants
    pub fn lexer(input: String) -> Result<Vec<tokens::Tokens>, &'static str> {

        //Declarations for the lexer's operation.
        let mut scope_stack: Vec<tokens::Tokens> = Vec::new();  //Declare a vector functioning as a stack for handling scope. The type of token in the stack determines the head's reading mode.
        let mut contents_stack: String = String::new();         //Declare a vector to hold contents that have been collected.
        let mut comments_stack: String = String::new();         //Declare a vector to hold comments that have been collected. These have to be separate since the tokens are different.
        #[derive(Debug)]
        enum Mode {                                             //Define an enum for the head's operating mode.
            Keys,     // The head is reading for diazo keywords
            Contents, // The head is reading for contents which don't have functions in the language
            Comments, // The head is reading comments and ignoring them
        }
        let mut mode: Mode;                                     //Declare a variable to store the current mode.
        let mut types: Vec<String> = Vec::new();                //Initialize a vector to store the types that the lexer can recognize.
        let mut output:Vec<tokens::Tokens> = Vec::new();        //Initialize output vector.
        let mut line_num: usize = 0;                            //Declare line number counter for debugging output.
        let mut word_num: usize;                            //Declare word number counter for debugging output.
    
        //Local function for returning the location of a syntax error.
        fn error_locator(a: usize, b: usize, c: &str) {
            eprintln!("Issue found at line {}, word {}, token:{}", a, b, c);
        }
        //Local function for checking if a token is in a position in a vector and returning a bool based on that.
        fn logic_check(v: &Vec<tokens::Tokens>, i: usize, t: tokens::Tokens) -> bool {
            if v.len() >= i && (v.len() - i) != 0 {                             //First we check if we can index into the vector.
                return *v.get(v.len() - 1 - i).unwrap() == t //Then compare what we find with our desired token t.
            }                                                   //Parameter i is the number of places before the end of the vector we are looking at.
            false                                               //If the vector is empty, of course we cannot match t.
        }
        //Local function for multi-threaded file-handling.
        fn import_file(filename: &String, output_to_edit: &mut Vec<tokens::Tokens>, typelist_to_edit: &mut Vec<String>) -> Result<(), &'static str> {
            //Find a file and read it, or else report that an issue has occurred.
            let s = match fs::read_to_string(&filename) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error occurred while attempting to import a file: {}", e);
                    return Err("Error, see above. ^^");
                },
            };
            //Lex the file.
            let (temp1, temp2) = match abridged_lexer(s, &filename) {
                Ok((v, t)) => (v, t),
                Err(e) => {
                    eprintln!("Error occurred while attempting to lex an imported file: {}", e);
                    return Err("Error, see above. ^^");
                }
            };
            for i in temp1 {
                output_to_edit.push(i);
            }
            for i in temp2 {
                typelist_to_edit.push(i);
            }
            Ok(())
        }

        for l in input.lines() {                          //Iterate over input linewise.
            //At the start of each line, reset the head to handle keywords unless a block comment is active.
            mode = if logic_check(&scope_stack, 0, tokens::Tokens::CommentOpen) {
                if comments_stack.len() != 0 {    //Clear out any comments from the last line.
                    output.push(tokens::Tokens::CommentContents(comments_stack));
                    if !logic_check(&output, 0, tokens::Tokens::Linebreak) {output.push(tokens::Tokens::Linebreak)};    //Place the linebreak back after our desired content.
                    comments_stack = String::new();
                }
                Mode::Comments                
            } else {
                if line_num > 1 {output.push(tokens::Tokens::Linebreak);}   //Add a linebreak at the start of each line but not the first.
                scope_stack.clear();
                //First check if anything was contained in the contents or comments strings, since switching out of those modes moves to Keys mode.
                if contents_stack.len() != 0 {    //Handle the text stack first, since contents always come before comments.
                    let t = logic_check(&output, 0, tokens::Tokens::Linebreak);
                    if t { output.pop(); }               //Remove the linebreak currently at the end of the output vector.
                    output.push(tokens::Tokens::Content(contents_stack));
                    if t { output.push(tokens::Tokens::Linebreak); }  //Place the linebreak back after our desired content.
                    contents_stack = String::new();
                }
                if comments_stack.len() != 0 {    //This is how inline comments are handled.
                    let t = logic_check(&output, 0, tokens::Tokens::Linebreak);
                    if t { output.pop(); }                 //Remove the linebreak currently at the end of the output vector.
                    output.push(tokens::Tokens::CommentContents(comments_stack));
                    if t { output.push(tokens::Tokens::Linebreak); }             //Place the linebreak back after our desired content.
                    comments_stack = String::new();
                }
                Mode::Keys
                
            };
            word_num = 0;                                       //Set the word counter to 0.
            let mut line_scope_counter: usize = 0;              //Declare a local scope counter.
            line_num += 1;                                      //Increment the line counter which will be returned in error messages.

            'words: for w in l.replace("\t", " *tab! ").split_whitespace() {     //Replace hard tab characters with a keyword, iterate over separated whitespaces.
                word_num += 1;  //Increment the word counter which will be returned in error messages.
                match mode {
                    Mode::Keys => {
                        //Once that's done, get to work matching different tokens.
                        match w {
                            "*tab!" => {                            //Handle tabs, which are used to determine scope.
                                if word_num == 1 || output.get(output.len() - 1) == Some(&tokens::Tokens::Tab) {
                                    line_scope_counter += 1;        //Increment scope counter only if tabs are at the start of the line.
                                }
                                output.push(tokens::Tokens::Tab);   //Add to output vector.
                            },
                            "//" => {                                       //Line Comments
                                mode = Mode::Comments;                      //Switch head mode to comments.
                                output.push(tokens::Tokens::CommentLine);   //Push the comment symbol to the output vector.
                            },
                            "/*" => {                                             //Open Block Comments
                                mode = Mode::Comments;
                                scope_stack.push(tokens::Tokens::CommentOpen);    //Push the comment block symbol to the scope stack. 
                                output.push(tokens::Tokens::CommentOpen);         // ^ Its presence stops the head from switching back to Keys mode after each line.
                            },
                            "*type" => {    //Keyword for declaring new types
                                //Check if type has appeared in a scope somewhere, which it really shouldn't.
                                if line_scope_counter != 0 {
                                    error_locator(line_num, word_num, w);
                                    return Err("Invalid syntax: *type has been placed in a scope somewhere, it should not have been. There is no place for a type declaration in a scope. (Remove any tabs before this *type)");
                                }
                                scope_stack.push(tokens::Tokens::TypeKeyword);  //Push the keyword to the scope stack. 
                                output.push(tokens::Tokens::TypeKeyword);       // ^ Its presence allows for handling of new names for types in the "other" arm.
                            },
                            "*use" => { //Keyword for noting that another file's declarations will be used.
                                //Check for a variety of errors that can occur with the keyword's use.
                                if logic_check(&scope_stack, 0, tokens::Tokens::UseKeyword) {
                                    error_locator(line_num, word_num, w);
                                    return Err("Invalid syntax: *use has already been called, you cannot have multiple on the same line.");
                                } else if line_scope_counter != 0 {
                                    error_locator(line_num, word_num, w);
                                    return Err("Invalid syntax: *use has been placed in a scope somewhere, it should not have been. There is no place for an import in a scope. (Remove any tabs before this *use)");
                                } else if logic_check(&scope_stack, 0, tokens::Tokens::Assignment) || logic_check(&scope_stack, 0, tokens::Tokens::TypeKeyword) {
                                    error_locator(line_num, word_num, w);
                                    return Err("Invalid syntax: *use has been called in a type declaration. You can't do this.");
                                }
                                scope_stack.push(tokens::Tokens::UseKeyword);   //Place the *use into the scope stack. The next token must be a filename becuase of this.
                                output.push(tokens::Tokens::UseKeyword);
                            },
                            "=>" => {
                                /* Check that the assignment is being used properly
                                    It might be used without an accompanying type keyword,
                                    or multiple words might have been put in place of the type's name being assigned */
                                if !logic_check(&scope_stack, 0, tokens::Tokens::TypeKeyword) {
                                    error_locator(line_num, word_num, w);
                                    return Err("Invalid syntax: Assignment operator used somewhere other than a type assignment.");
                                } else if logic_check(&scope_stack, 0, tokens::Tokens::TypeKeyword) && !logic_check(&output, 1, tokens::Tokens::TypeKeyword) {
                                    error_locator(line_num, word_num - 2, w);
                                    return Err("Invalid syntax: Type names should only be one word. Try using underscores, or check the declaration.");
                                }
                                scope_stack.push(tokens::Tokens::Assignment);
                                output.push(tokens::Tokens::Assignment);
                            },
                            other => {
                                //Check if *use has been called and is immediately before this word, in which case we are dealing with a filename.
                                if logic_check(&scope_stack, 0, tokens::Tokens::UseKeyword) && logic_check(&output, 0, tokens::Tokens::UseKeyword) {
                                    scope_stack.pop();  //Remove the *use keyword from the scope_stack
                                    output.push(tokens::Tokens::Filename(other.to_string()));
                                    output.push(tokens::Tokens::Linebreak);
                                    match import_file(&other.to_string(), &mut output, &mut types) {
                                        Ok(_) => (),
                                        Err(e) => {
                                            eprintln!("Error occurred while attempting to lex an imported file: {}", e);
                                            return Err("Error, see above. ^^");
                                        }
                                    };
                                    continue 'words
                                }
                                //Check if we are immediately after a type keyword but before the arrow.
                                if logic_check(&output, 0, tokens::Tokens::TypeKeyword) && logic_check(&scope_stack, 0, tokens::Tokens::TypeKeyword) {
                                    if types.contains(&other.clone().to_string()) {             //Prevent type declarations to the same name.
                                        error_locator(line_num, word_num, w);
                                        return Err("Invalid syntax: It appears this type has been declared before, the namespace is already occupied!");
                                    }
                                    types.push(other.clone().to_string());                      //Add the new type as a valid option for use in future code.
                                    output.push(tokens::Tokens::TypeName(other.to_string()));   //Add the token onto the output vector too.
                                    continue
                                }
                                
                                //Check if we are in the arguments section of a type declaration.
                                if logic_check(&scope_stack, 1, tokens::Tokens::TypeKeyword) &&    //The second to last item in the scope_stack is the TypeKeyword.
                                    logic_check(&scope_stack, 0, tokens::Tokens::Assignment) {      //The last item in the scope_stack was an assignment symbol.
                                    if other == "any" {
                                        output.push(tokens::Tokens::Any);
                                        continue
                                    }
                                    if types.contains(&String::from(other.clone())) {
                                        output.push(tokens::Tokens::TypeAsDeclarationParameter(String::from(other)));
                                        continue
                                    }
                                    match other.clone().chars().nth(0).unwrap() {
                                        'e' => {
                                            let a = other.clone();  //Declare clone of other for checking around with logic.
                                            //Make sure that there is a .. in between the e and whatever follows. Sadly we cannot check if there is an n or number yet.
                                            if a.len() > 1 && (a.chars().nth(1).unwrap(), a.chars().nth(2).unwrap()) != ('.', '.') {
                                                error_locator(line_num, word_num, w);
                                                return Err("Invalid syntax: Something besides \"..\" is separating the e from the number/letter here.
                                                            Alternatively there's something else entirely following the e");
                                            }
                                            output.push(tokens::Tokens::E(other.to_string()));
                                            continue
                                        },
                                        'c' => {
                                            let a = other.clone();  //Declare clone of other for checking around with logic.
                                            //Make sure that there is a .. in between the c and whatever follows. Sadly we cannot check if there is an n or number yet.
                                            if a.len() > 1 && (a.chars().nth(1).unwrap(), a.chars().nth(2).unwrap()) != ('.', '.') {
                                                error_locator(line_num, word_num, w);
                                                return Err("Invalid syntax: Something besides \"..\" is separating the c from the number/letter here.
                                                            Alternatively there's something else entirely following the c");
                                            }
                                            output.push(tokens::Tokens::C(other.to_string()));
                                            continue
                                        },
                                        _other => {  //The only things that can be in the arguments section are c, e, any, or a type that's been declared.
                                        error_locator(line_num, word_num, w);
                                            return Err("Invalid syntax: Something other than a type name, \"e\", \"c\", or \"any\".
                                                        If it is a type name, it hasn't been declared yet and so cannot be recognized.");
                                        }
                                    }
                                }
                                //Check if a comment was opened but there was no space between the comment symbol and the following word. (As in this line)
                                if other.contains("//") {
                                    output.push(tokens::Tokens::CommentLine);
                                    comments_stack.push_str(other.replace("//", "").as_str());
                                    mode = Mode::Comments;
                                    continue
                                } else if other.contains("/*") {
                                    output.push(tokens::Tokens::CommentOpen);
                                    comments_stack.push_str(other.replace("/*", "").as_str());
                                    mode = Mode::Comments;
                                    scope_stack.push(tokens::Tokens::CommentOpen);
                                    continue
                                }
                                //Otherwise, we are probably reading for types to be instantiated. Check if we're reading a type that's been declared.
                                if types.contains(&other.clone().to_string()) {
                                    output.push(tokens::Tokens::TypeInstance(other.to_string()));
                                    mode = Mode::Contents;  //We've just identified that a type has been instantiated. This transitions the head to read for contents instead.
                                } else {
                                //If we can't find a type name, we are already writing content!
                                //First we do the usual check to make sure the word doesn't end in a separator...
                                for i in tokens::separator_list() {     //Compare it to the list of separators... only works on 2 character-long seperators.
                                    if other.contains(i) {
                                        contents_stack.push_str(other.replace(i, " ").as_str());    //Terminate the current contents stack.
                                        output.push(tokens::Tokens::Content(contents_stack.clone()));            //Send it to the output stack.
                                        contents_stack.clear();                                                  //Clear contents stack.
                                        output.push(tokens::Tokens::Separator(String::from(i)));                 //Send the separator to the output stack.
                                        continue 'words
                                    }
                                }
                                //Also check that we haven't tried to start a math or code formatting block...
                                if other == "[[" {
                                    output.push(tokens::Tokens::CodeBlockOpen);
                                    mode = Mode::Contents;
                                    continue 'words
                                } else if other == "{{" {
                                    output.push(tokens::Tokens::MathBlockOpen);
                                    mode = Mode::Contents;
                                    continue 'words
                                }
                                //Finally, send the word to the content stack and switch head mode to content.
                                contents_stack.push_str(other);
                                contents_stack.push(' ');
                                mode = Mode::Contents;
                                }
                            },
                        }
                    },
                    Mode::Contents => {
                        //There are a few things we can encounter once we have begun reading in contents mode.
                        //First, symbols for controlling formatting blocks.
                        if w == "[[" {
                            output.push(tokens::Tokens::Content(contents_stack.clone()));   //Interrupt the contents stack and send it to the output stack.
                            contents_stack.clear();                                         //Clear contents stack.
                            output.push(tokens::Tokens::CodeBlockOpen);                     //Push the appropriate symbol to the output stack.
                            continue 'words                                                 //Continue reading the contents.
                        } else if w == "]]" {
                            output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            output.push(tokens::Tokens::CodeBlockClose);
                            continue 'words
                        } else if w == "{{" {
                            output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            output.push(tokens::Tokens::MathBlockOpen);
                            continue 'words
                        } else if w == "}}" {
                            output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            output.push(tokens::Tokens::MathBlockClose);
                            continue 'words
                        }

                        //Unfortunately, it is also possible that the user has written their text with the formatting symbols attached to words e.g. [[example
                        if w.contains("[[") {
                            output.push(tokens::Tokens::Content(contents_stack.clone()));           //Interrupt the contents stack and send it to the output stack.
                            contents_stack.clear();                                                 //Clear the contents stack.
                            output.push(tokens::Tokens::CodeBlockOpen);                             //Send the symbol to the output stack.
                            contents_stack.push_str(w.replace("[[", "").as_str()); //Add whatever follows the symbol to the contents stack.
                            continue 'words
                        } else if w.contains("]]") {                  //The procedure is slightly different for the end symbols. Not only is the order
                            for i in tokens::separator_list() { //in which the default procedure is executed different, but we must also consider the possibility of both content and a separator being attached to the symbol.
                                if w.contains(i) {                                                                                      //Check each of the possible separators.
                                    contents_stack.push_str(w.replace("]]", "").replace(i, " ").as_str());    //Terminate the current contents stack.
                                    output.push(tokens::Tokens::Content(contents_stack.clone()));                                       //Send it to the output stack.
                                    contents_stack.clear();                                                                             //Clear contents stack.
                                    output.push(tokens::Tokens::CodeBlockClose);                                                        //Send the end block symbol to the output stack.
                                    output.push(tokens::Tokens::Separator(String::from(i)));                                            //Send the separator to the output stack.
                                    continue 'words
                                }
                            }                                                                        //Procedure if there is no separator mixed in.
                            contents_stack.push_str(w.replace("]]", " ").as_str()); //Terminate the current contents stack.
                            output.push(tokens::Tokens::Content(contents_stack.clone()));            //Send it to the output stack.
                            contents_stack.clear();                                                  //Clear contents stack.
                            output.push(tokens::Tokens::CodeBlockClose);                             //Send the symbol to the output stack.
                            continue 'words
                        } else if w.contains("{{") {
                            output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            output.push(tokens::Tokens::MathBlockOpen);
                            contents_stack.push_str(w.replace("{{", "").as_str());
                            continue 'words
                        } else if w.contains("}}") {    //Similar procedure to the end code block symbol.
                            for i in tokens::separator_list() {
                                if w.contains(i) {
                                    contents_stack.push_str(w.replace("}}", "").replace(i, " ").as_str());
                                    output.push(tokens::Tokens::Content(contents_stack.clone()));
                                    contents_stack.clear();
                                    output.push(tokens::Tokens::MathBlockClose);
                                    output.push(tokens::Tokens::Separator(String::from(i)));
                                    continue 'words
                                }
                            }
                            contents_stack.push_str(w.replace("]]", " ").as_str());
                            output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            output.push(tokens::Tokens::MathBlockClose);
                            continue 'words
                        }

                        //Alternatively, a separator, of which there are some varieties but all have the same functionality. They just look different.
                        for i in tokens::separator_list() {
                            if i == w {
                                output.push(tokens::Tokens::Separator(w.to_string()));
                            }
                        }
                        //If we aren't dealing with a separator, we could be loading things into the contents String. But it might have a separator attached so let's clear that up too.
                        for i in tokens::separator_list() {     //Compare it to the list of separators... only works on 2 character-long separators.
                            if w.contains(i) {
                                contents_stack.push_str(w.replace(i, " ").as_str());    //Terminate the current contents stack.
                                output.push(tokens::Tokens::Content(contents_stack.clone()));            //Send it to the output stack.
                                contents_stack.clear();                                                  //Clear contents stack.
                                output.push(tokens::Tokens::Separator(String::from(i)));                 //Send the separator to the output stack.
                                continue 'words
                            }
                        }
                        contents_stack.push_str(w);
                        contents_stack.push(' ');
                    },
                    Mode::Comments => {
                        //There are only two things we can encounter once we have started reading in comments mode.
                        //The end comment block symbol, which pops the open comment block out of the scope stack and transitions the head to keys mode.
                        if w == "*/" {
                            if logic_check(&scope_stack, 0, tokens::Tokens::CommentOpen) {
                                scope_stack.pop();
                                mode = Mode::Keys;
                                continue
                            } else {
                                error_locator(line_num, word_num, w);
                                return Err("Invalid syntax: You've tried to close a block comment but this isn't a block comment!");
                            }
                        }
                        //Check that the word doesn't end in the end comment block symbol, which it could if no space has been placed there.
                        if w.contains("*/") {
                            if logic_check(&scope_stack, 0, tokens::Tokens::CommentOpen) {
                                comments_stack.push_str(w.replace("*/", " ").as_str()); //Terminate the current comments stack.
                                output.push(tokens::Tokens::CommentContents(comments_stack.clone()));   //Send it to the output stack.
                                comments_stack.clear();                                                 //Clear comments stack.
                                scope_stack.pop();
                                mode = Mode::Keys;
                                continue
                            } else {
                                error_locator(line_num, word_num, w);
                                return Err("Invalid syntax: You've tried to close a block comment but this isn't a block comment!");
                            }
                        }
                        //Alternatively, comments, which are loaded into a String.
                        comments_stack.push_str(w);
                        comments_stack.push(' ');
                    },
                }
            }
        }
        //Check if the comments or contents stacks are empty or not. It there is something there, empty it out.
        if contents_stack.len() != 0 {    //Technically these two situations should be mutually exclusive.
            output.push(tokens::Tokens::Content(contents_stack));
        }
        if comments_stack.len() != 0 {
            output.push(tokens::Tokens::CommentContents(comments_stack));
        }
        Ok(output)  //Since everything has been okay so far, return the output vector wrapped in Ok!
    }

    //Abridged lexer, which cannot perform filehandling (no imports). Prevents recursive behavior (a file imports another file which imports another file).
    //This version of the lexer is called in the new threads when importing other files.
    //It also only places type declarations into the output.
    fn abridged_lexer(input: String, filename: &String) -> Result<(Vec<tokens::Tokens>, Vec<String>), &'static str> {

        //Declarations for the lexer's operation.
        let mut scope_stack: Vec<tokens::Tokens> = Vec::new();  //Declare a vector functioning as a stack for handling scope. The type of token in the stack determines the head's reading mode.
        let mut contents_stack: String = String::new();         //Declare a vector to hold contents that have been collected.
        let mut comments_stack: String = String::new();         //Declare a vector to hold comments that have been collected. These have to be separate since the tokens are different.
        #[derive(Debug)]
        enum Mode {                                             //Define an enum for the head's operating mode.
            Keys,     // The head is reading for diazo keywords
            Contents, // The head is reading for contents which don't have functions in the language
            Comments, // The head is reading comments and ignoring them
        }
        let mut mode: Mode;                                     //Declare a variable to store the current mode.
        let mut types: Vec<String> = Vec::new();                //Initialize a vector to store the types that the lexer can recognize.
        let mut syntax_check_output:Vec<tokens::Tokens> = Vec::new();       //Initialize an output vector which is used for ensuring that the imported file has enough good syntax to be trustworthy.
        let mut final_output: Vec<tokens::Tokens> = Vec::new();             //Initialize output that will actually be used.
        let mut line_num: usize = 0;                            //Declare line number counter for debugging output.
        let mut word_num: usize;                                //Declare word number counter for debugging output.
    
        //Local function for returning the location of a syntax error.
        fn error_locator(a: &String, b: usize, c: usize, d: &str) {
            eprintln!("Issue in imported file: {}.\nIssue found at line {}, word {}, token:{}", a, b, c, d);
        }
        //Local function for checking if a token is in a position in a vector and returning a bool based on that.
        fn logic_check(v: &Vec<tokens::Tokens>, i: usize, t: tokens::Tokens) -> bool {
            if v.len() >= i && (v.len() - i) != 0 {                             //First we check if we can index into the vector.
                return *v.get(v.len() - 1 - i).unwrap() == t //Then compare what we find with our desired token t.
            }                                                   //Parameter i is the number of places before the end of the vector we are looking at.
            false                                               //If the vector is empty, of course we cannot match t.
        }

        for l in input.lines() {                          //Iterate over input linewise.
            //At the start of each line, reset the head to handle keywords unless a block comment is active.
            mode = if logic_check(&scope_stack, 0, tokens::Tokens::CommentOpen) {
                if comments_stack.len() != 0 {    //Clear out any comments from the last line.
                    syntax_check_output.push(tokens::Tokens::CommentContents(comments_stack));
                    if !logic_check(&syntax_check_output, 0, tokens::Tokens::Linebreak) {syntax_check_output.push(tokens::Tokens::Linebreak)};    //Place the linebreak back after our desired content.
                    comments_stack = String::new();
                }
                Mode::Comments                
            } else {
                if line_num > 1 {syntax_check_output.push(tokens::Tokens::Linebreak);}   //Add a linebreak at the start of each line but not the first.
                if scope_stack.contains(&tokens::Tokens::TypeKeyword) { //Check if we just came from a type declaration.
                    final_output.push(tokens::Tokens::Linebreak);       //In which case, insert a line break.
                }
                scope_stack.clear();
                //First check if anything was contained in the contents or comments strings, since switching out of those modes moves to Keys mode.
                if contents_stack.len() != 0 {    //Handle the text stack first, since contents always come before comments.
                    let t = logic_check(&syntax_check_output, 0, tokens::Tokens::Linebreak);
                    if t { syntax_check_output.pop(); }               //Remove the linebreak currently at the end of the output vector.
                    syntax_check_output.push(tokens::Tokens::Content(contents_stack));
                    if t { syntax_check_output.push(tokens::Tokens::Linebreak); }  //Place the linebreak back after our desired content.
                    contents_stack = String::new();
                }
                if comments_stack.len() != 0 {    //This is how inline comments are handled.
                    let t = logic_check(&syntax_check_output, 0, tokens::Tokens::Linebreak);
                    if t { syntax_check_output.pop(); }                 //Remove the linebreak currently at the end of the output vector.
                    syntax_check_output.push(tokens::Tokens::CommentContents(comments_stack));
                    if t { syntax_check_output.push(tokens::Tokens::Linebreak); }             //Place the linebreak back after our desired content.
                    comments_stack = String::new();
                }
                Mode::Keys
            };
            word_num = 0;                                       //Set the word counter to 0.
            let mut line_scope_counter: usize = 0;              //Declare a local scope counter.
            line_num += 1;                                      //Increment the line counter which will be returned in error messages.

            'words: for w in l.replace("\t", " *tab! ").split_whitespace() {     //Replace hard tab characters with a keyword, iterate over separated whitespaces.
                word_num += 1;  //Increment the word counter which will be returned in error messages.
                match mode {
                    Mode::Keys => {
                        //Once that's done, get to work matching different tokens.
                        match w {
                            "*tab!" => {                            //Handle tabs, which are used to determine scope.
                                if word_num == 1 || syntax_check_output.get(syntax_check_output.len() - 1) == Some(&tokens::Tokens::Tab) {
                                    line_scope_counter += 1;                        //Increment scope counter,
                                    syntax_check_output.push(tokens::Tokens::Tab);  //and add the tab to the tokens list only if they are at the start of the line.
                                }
                            },
                            "//" => {                                       //Line Comments
                                mode = Mode::Comments;                      //Switch head mode to comments.
                                syntax_check_output.push(tokens::Tokens::CommentLine);   //Push the comment symbol to the output vector.
                            },
                            "/*" => {                                             //Open Block Comments
                                mode = Mode::Comments;
                                scope_stack.push(tokens::Tokens::CommentOpen);    //Push the comment block symbol to the scope stack. 
                                syntax_check_output.push(tokens::Tokens::CommentOpen);         // ^ Its presence stops the head from switching back to Keys mode after each line.
                            },
                            "*type" => {    //Keyword for declaring new types
                                //Check if type has appeared in a scope somewhere, which it really shouldn't.
                                if line_scope_counter != 0 {
                                    error_locator(filename, line_num, word_num, w);
                                    return Err("Invalid syntax: *type has been placed in a scope somewhere, it should not have been. There is no place for a type declaration in a scope. (Remove any tabs before this *type)");
                                }
                                scope_stack.push(tokens::Tokens::TypeKeyword);  //Push the keyword to the scope stack. 
                                syntax_check_output.push(tokens::Tokens::TypeKeyword);       // ^ Its presence allows for handling of new names for types in the "other" arm.
                                final_output.push(tokens::Tokens::TypeKeyword); //Push keyword into the final output which is what is visible to the calling lexer.
                            },
                            "=>" => {
                                /* Check that the assignment is being used properly
                                    It might be used without an accompanying type keyword,
                                    or multiple words might have been put in place of the type's name being assigned */
                                if !logic_check(&scope_stack, 0, tokens::Tokens::TypeKeyword) {
                                    error_locator(filename, line_num, word_num, w);
                                    return Err("Invalid syntax: Assignment operator used somewhere other than a type assignment.");
                                } else if logic_check(&scope_stack, 0, tokens::Tokens::TypeKeyword) && !logic_check(&syntax_check_output, 1, tokens::Tokens::TypeKeyword) {
                                    error_locator(filename, line_num, word_num - 2, w);
                                    return Err("Invalid syntax: Type names should only be one word. Try using underscores, or check the declaration.");
                                }
                                scope_stack.push(tokens::Tokens::Assignment);
                                syntax_check_output.push(tokens::Tokens::Assignment);
                                final_output.push(tokens::Tokens::Assignment); //Push assignment symbol into the final output which is what is visible to the calling lexer.
                            },
                            other => {
                                //Check if we are immediately after a type keyword but before the arrow.
                                if logic_check(&syntax_check_output, 0, tokens::Tokens::TypeKeyword) && logic_check(&scope_stack, 0, tokens::Tokens::TypeKeyword) {
                                    if types.contains(&other.clone().to_string()) {             //Prevent type declarations to the same name.
                                        error_locator(filename, line_num, word_num, w);
                                        return Err("Invalid syntax: It appears this type has been declared before, the namespace is already occupied!");
                                    }
                                    types.push(other.clone().to_string());                      //Add the new type as a valid option for use in future code.
                                    syntax_check_output.push(tokens::Tokens::TypeName(other.to_string()));   //Add the token onto the output vector too.
                                    final_output.push(tokens::Tokens::TypeName(other.to_string())); //Push new typename into the final output which is what is visible to the calling lexer.
                                    continue
                                }
                                
                                //Check if we are in the arguments section of a type declaration.
                                if logic_check(&scope_stack, 1, tokens::Tokens::TypeKeyword) &&    //The second to last item in the scope_stack is the TypeKeyword.
                                    logic_check(&scope_stack, 0, tokens::Tokens::Assignment) {      //The last item in the scope_stack was an assignment symbol.
                                    if other == "any" {
                                        syntax_check_output.push(tokens::Tokens::Any);
                                        final_output.push(tokens::Tokens::Any); //Push keyword into the final output which is what is visible to the calling lexer.
                                        continue
                                    }
                                    if types.contains(&String::from(other.clone())) {
                                        syntax_check_output.push(tokens::Tokens::TypeAsDeclarationParameter(String::from(other)));
                                        final_output.push(tokens::Tokens::TypeAsDeclarationParameter(String::from(other)));
                                        continue
                                    }
                                    match other.clone().chars().nth(0).unwrap() {
                                        'e' => {
                                            let a = other.clone();  //Declare clone of other for checking around with logic.
                                            //Make sure that there is a .. in between the e and whatever follows. Sadly we cannot check if there is an n or number yet.
                                            if a.len() > 1 && (a.chars().nth(1).unwrap(), a.chars().nth(2).unwrap()) != ('.', '.') {
                                                error_locator(filename, line_num, word_num, w);
                                                return Err("Invalid syntax: Something besides \"..\" is separating the e from the number/letter here.
                                                            Alternatively there's something else entirely following the e");
                                            }
                                            syntax_check_output.push(tokens::Tokens::E(other.to_string()));
                                            final_output.push(tokens::Tokens::E(other.to_string()));
                                            continue
                                        },
                                        'c' => {
                                            let a = other.clone();  //Declare clone of other for checking around with logic.
                                            //Make sure that there is a .. in between the c and whatever follows. Sadly we cannot check if there is an n or number yet.
                                            if a.len() > 1 && (a.chars().nth(1).unwrap(), a.chars().nth(2).unwrap()) != ('.', '.') {
                                                error_locator(filename, line_num, word_num, w);
                                                return Err("Invalid syntax: Something besides \"..\" is separating the c from the number/letter here.
                                                            Alternatively there's something else entirely following the c");
                                            }
                                            syntax_check_output.push(tokens::Tokens::C(other.to_string()));
                                            final_output.push(tokens::Tokens::C(other.to_string()));
                                            continue
                                        },
                                        _other => {  //The only things that can be in the arguments section are c, e, any, or a type that's been declared.
                                        error_locator(filename, line_num, word_num, w);
                                            return Err("Invalid syntax: Something other than a type name, \"e\", \"c\", or \"any\".
                                                        If it is a type name, it hasn't been declared yet and so cannot be recognized.");
                                        }
                                    }
                                }
                                //Check if a comment was opened but there was no space between the comment symbol and the following word. (As in this line)
                                if other.contains("//") {
                                    syntax_check_output.push(tokens::Tokens::CommentLine);
                                    comments_stack.push_str(other.replace("//", "").as_str());
                                    mode = Mode::Comments;
                                    continue
                                } else if other.contains("/*") {
                                    syntax_check_output.push(tokens::Tokens::CommentOpen);
                                    comments_stack.push_str(other.replace("/*", "").as_str());
                                    mode = Mode::Comments;
                                    scope_stack.push(tokens::Tokens::CommentOpen);
                                    continue
                                }
                                //Otherwise, we are probably reading for types to be instantiated. Check if we're reading a type that's been declared.
                                if types.contains(&other.clone().to_string()) {
                                    syntax_check_output.push(tokens::Tokens::TypeInstance(other.to_string()));
                                    mode = Mode::Contents;  //We've just identified that a type has been instantiated. This transitions the head to read for contents instead.
                                } else {
                                //If we can't find a type name, we are already writing content!
                                //First we do the usual check to make sure the word doesn't end in a separator...
                                for i in tokens::separator_list() {     //Compare it to the list of separators... only works on 2 character-long seperators.
                                    if other.contains(i) {
                                        contents_stack.push_str(other.replace(i, " ").as_str());    //Terminate the current contents stack.
                                        syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));   //Send it to the output stack.
                                        contents_stack.clear();                                                      //Clear contents stack.
                                        syntax_check_output.push(tokens::Tokens::Separator(String::from(i)));        //Send the separator to the output stack.
                                        continue 'words
                                    }
                                }
                                //Also check that we haven't tried to start a math or code formatting block...
                                if other == "[[" {
                                    syntax_check_output.push(tokens::Tokens::CodeBlockOpen);
                                    mode = Mode::Contents;
                                    continue 'words
                                } else if other == "{{" {
                                    syntax_check_output.push(tokens::Tokens::MathBlockOpen);
                                    mode = Mode::Contents;
                                    continue 'words
                                }
                                //Finally, send the word to the content stack and switch head mode to content.
                                contents_stack.push_str(other);
                                contents_stack.push(' ');
                                mode = Mode::Contents;
                                }
                            },
                        }
                    },
                    Mode::Contents => {
                        //There are two things we can encounter once we have begun reading in contents mode.
                        //First, symbols for controlling formatting blocks.
                        if w == "[[" {
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));   //Interrupt the contents stack and send it to the output stack.
                            contents_stack.clear();                                                      //Clear contents stack.
                            syntax_check_output.push(tokens::Tokens::CodeBlockOpen);                     //Push the appropriate symbol to the output stack.
                            continue 'words                                                              //Continue reading the contents.
                        } else if w == "]]" {
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            syntax_check_output.push(tokens::Tokens::CodeBlockClose);
                            continue 'words
                        } else if w == "{{" {
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            syntax_check_output.push(tokens::Tokens::MathBlockOpen);
                            continue 'words
                        } else if w == "}}" {
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            syntax_check_output.push(tokens::Tokens::MathBlockClose);
                            continue 'words
                        }

                        //Unfortunately, it is also possible that the user has written their text with the formatting symbols attached to words e.g. [[example
                        if w.contains("[[") {
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));  //Interrupt the contents stack and send it to the output stack.
                            contents_stack.clear();                                                     //Clear the contents stack.
                            syntax_check_output.push(tokens::Tokens::CodeBlockOpen);                    //Send the symbol to the output stack.
                            contents_stack.push_str(w.replace("[[", "").as_str());     //Add whatever follows the symbol to the contents stack.
                            continue 'words
                        } else if w.contains("]]") {                  //The procedure is slightly different for the end symbols. Not only is the order
                            for i in tokens::separator_list() { //in which the default procedure is executed different, but we must also consider the possibility of both content and a separator being attached to the symbol.
                                if w.contains(i) {                                                                                      //Check each of the possible separators.
                                    contents_stack.push_str(w.replace("]]", "").replace(i, " ").as_str());    //Terminate the current contents stack.
                                    syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));                          //Send it to the output stack.
                                    contents_stack.clear();                                                                             //Clear contents stack.
                                    syntax_check_output.push(tokens::Tokens::CodeBlockClose);                                           //Send the end block symbol to the output stack.
                                    syntax_check_output.push(tokens::Tokens::Separator(String::from(i)));                               //Send the separator to the output stack.
                                    continue 'words
                                }
                            }                                                                           //Procedure if there is no separator mixed in.
                            contents_stack.push_str(w.replace("]]", " ").as_str());    //Terminate the current contents stack.
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));  //Send it to the output stack.
                            contents_stack.clear();                                                     //Clear contents stack.
                            syntax_check_output.push(tokens::Tokens::CodeBlockClose);                   //Send the symbol to the output stack.
                            continue 'words
                        } else if w.contains("{{") {
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            syntax_check_output.push(tokens::Tokens::MathBlockOpen);
                            contents_stack.push_str(w.replace("{{", "").as_str());
                            continue 'words
                        } else if w.contains("}}") {    //Similar procedure to the end code block symbol.
                            for i in tokens::separator_list() {
                                if w.contains(i) {
                                    contents_stack.push_str(w.replace("}}", "").replace(i, " ").as_str());
                                    syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));
                                    contents_stack.clear();
                                    syntax_check_output.push(tokens::Tokens::MathBlockClose);
                                    syntax_check_output.push(tokens::Tokens::Separator(String::from(i)));
                                    continue 'words
                                }
                            }
                            contents_stack.push_str(w.replace("]]", " ").as_str());
                            syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));
                            contents_stack.clear();
                            syntax_check_output.push(tokens::Tokens::MathBlockClose);
                            continue 'words
                        }

                        //Alternatively, a separator, of which there are some varieties but all have the same functionality. They just look different.
                        for i in tokens::separator_list() {
                            if i == w {
                                syntax_check_output.push(tokens::Tokens::Separator(w.to_string()));
                            }
                        }
                        //If we aren't dealing with a separator we are loading things into the contents String. But it might have a separator attached so let's clear that up too.
                        for i in tokens::separator_list() {     //Compare it to the list of separators... only works on 2 character-long separators.
                            if w.contains(i) {
                                contents_stack.push_str(w.replace(i, " ").as_str());    //Terminate the current contents stack.
                                syntax_check_output.push(tokens::Tokens::Content(contents_stack.clone()));            //Send it to the output stack.
                                contents_stack.clear();                                                  //Clear contents stack.
                                syntax_check_output.push(tokens::Tokens::Separator(String::from(i)));                 //Send the separator to the output stack.
                                continue 'words
                            }
                        }
                        contents_stack.push_str(w);
                        contents_stack.push(' ');
                    },
                    Mode::Comments => {
                        //Similarly, there are only two things we can encounter once we have started reading in contents mode.
                        //The end comment block symbol, which pops the open comment block out of the scope stack and transitions the head to keys mode.
                        if w == "*/" {
                            if logic_check(&scope_stack, 0, tokens::Tokens::CommentOpen) {
                                scope_stack.pop();
                                mode = Mode::Keys;
                                continue
                            } else {
                                error_locator(filename, line_num, word_num, w);
                                return Err("Invalid syntax: You've tried to close a block comment but this isn't a block comment!");
                            }
                        }
                        //Check that the word doesn't end in the end comment block symbol, which it could if no space has been placed there.
                        if w.contains("*/") {
                            if logic_check(&scope_stack, 0, tokens::Tokens::CommentOpen) {
                                comments_stack.push_str(w.replace("*/", " ").as_str()); //Terminate the current comments stack.
                                syntax_check_output.push(tokens::Tokens::CommentContents(comments_stack.clone()));   //Send it to the output stack.
                                comments_stack.clear();                                                 //Clear comments stack.
                                scope_stack.pop();
                                mode = Mode::Keys;
                                continue
                            } else {
                                error_locator(filename, line_num, word_num, w);
                                return Err("Invalid syntax: You've tried to close a block comment but this isn't a block comment!");
                            }
                        }
                        //Alternatively, comments, which are loaded into a String.
                        comments_stack.push_str(w);
                        comments_stack.push(' ');
                    },
                }
            }
        }
        //Check if the comments or contents stacks are empty or not. It there is something there, empty it out.
        if contents_stack.len() != 0 {    //Technically these two situations should be mutually exclusive.
            syntax_check_output.push(tokens::Tokens::Content(contents_stack));
        }
        if comments_stack.len() != 0 {
            syntax_check_output.push(tokens::Tokens::CommentContents(comments_stack));
        }
        Ok((final_output, types))  //Since everything has been okay so far, return the output vector wrapped in Ok!
    }
}

//Module containing the parser, a component designed to construct an abstract-syntax tree form the list of tokens.
pub mod parser {
    use std::{rc::Rc, cell::RefCell};
    use crate::core::{tokens, parser_structs};

    pub fn parser(mut input: Vec<tokens::Tokens>) -> Result<Vec<parser_structs::IrElements>, &'static str> {

        //Local function for returning the location of a syntax error.
        fn error_locator(a: usize, b: tokens::Tokens) {
            eprintln!("Issue found at line {}, token:{}", a, b.print());
        }

        //Local function to reduce boilerplate when filling the tree.
        fn tree_fill(fill_with: &tokens::Tokens) -> (Rc<RefCell<parser_structs::TreeNode>>, bool, tokens::Tokens) {
            (Rc::new(RefCell::new(parser_structs::TreeNode::new(fill_with.clone()))), true, fill_with.clone())
        }

        //Local function to reduce boilerplate when resetting the tree.
        fn tree_reset() -> (Rc<RefCell<parser_structs::TreeNode>>, bool, tokens::Tokens) {
            (Rc::new(RefCell::new(parser_structs::TreeNode::new(tokens::Tokens::Null))), false, tokens::Tokens::Null)
        }
        
        let mut output: Vec<parser_structs::IrElements> = Vec::new();   //Initialize a vector of IrElements, which represent's the parser's output.
        let mut tree: Rc<RefCell<parser_structs::TreeNode>> = Rc::new(RefCell::new(parser_structs::TreeNode::new(tokens::Tokens::Null)));  //Declare a variable namespace that holds the current tree.
        let mut tree_contains: bool = false;                                               //Variable that stores whether or not there is something worth reading in the tree.
        let mut token_in_tree: tokens::Tokens = tokens::Tokens::Null;   //Variable that stores what is in the tree.
        let mut formatting_stack: Vec<tokens::Tokens> = Vec::new();       //Vector that carries content being pooled for formatting.                  
        let mut previous: tokens::Tokens = tokens::Tokens::Null;            //Declare a variable to hold the previous token that was examined. Implementing look-ahead is either inefficient or requires external packages, so we will look back instead.
        let mut line: usize = 1;                                                                    //Initialize the line counter which will be used for error reporting.
        let mut last_line_scope: usize;                                                 //Variable that stores the previous line's scope counter.
        let mut line_scope: usize = 0;                                                          //Variable that stores the current line's scope counter.

        input.push(tokens::Tokens::Null);   //First push a null token to the end of the input since we are looking back at previous. (Would be one short otherwise).
        for i in input{ //Look through each of the tokens in the input.
            match &previous {
                tokens::Tokens::Null => {   //If there is nothing in the previous slot, move i into previous.
                    previous = i;   //Before advancing the loop, remember to move the current i into previous.
                },
                //Whitespace
                tokens::Tokens::Linebreak => {  //There are two possible outcomes when dealing with a linebreak...
                    last_line_scope = line_scope; //But in both of them, we will move the line_scope into the last_line_scope so that we can compare against the previous line.
                    if i == tokens::Tokens::Tab {         //First, there could be a tab after the linebreak,
                        if tree_contains {                       //We then check if an assignment or element operator is in the tree.
                            match token_in_tree {             //And return an error, since there should be nothing nested under a type declaration statement.
                                tokens::Tokens::Assignment | tokens::Tokens::E(_) => {
                                    error_locator(line + 1, i);
                                    return Err("Invalid syntax (Parser): Try removing this tab. Whitespace is functional and this tab is saying that what follows it is nested under the type declaration from the previous line.")
                                },
                                tokens::Tokens::ContentWithFormatting(_) | tokens::Tokens::TypeInstance(_) => {   //Two situations that are handled similarly, either leading to raw text expression or a type expression. (These are just formatting differences.)
                                    if line_scope >= last_line_scope && line_scope > 0 { //If the current line's scope is greater than the previous one, do nothing. We are indented and therefore still inside the same expression.
                                    } else {
                                        let mut switch = tree.borrow().parent.as_ref().is_some();   //Otherwise, check if the node has a parent.
                                        while switch {
                                            tree = parser_structs::TreeNode::back_to_parent(&tree);             //Go back to the node's parent as long as there is a parent.
                                            switch = tree.borrow().parent.as_ref().is_some();
                                            token_in_tree = tree.borrow().value.clone();
                                        }
                                        match token_in_tree {   //Since there is a possibility that we've switched the tree, check what the parent node we are dealing with is.
                                            tokens::Tokens::ContentWithFormatting(_) => {   //Raw content, in which the content is the root of the tree (likely not a full tree in this case).
                                                output.push(parser_structs::IrElements::RawText(Rc::clone(&tree))); //Now that there is no longer a parent node, we can push to the output vector.
                                                (tree, tree_contains, token_in_tree) = tree_reset();
                                            },
                                            tokens::Tokens::TypeInstance(_) => {
                                                output.push(parser_structs::IrElements::TypeExpression(Rc::clone(&tree)));
                                                (tree, tree_contains, token_in_tree) = tree_reset();
                                            },
                                            _ => {  //Otherwise return an error.
                                                error_locator(line, i);
                                                return Err("Invalid Syntax (Parser): Invalid tree structure... root node is neither formatting nor a type expression.")
                                            }
                                        }
                                    }
                                },
                                tokens::Tokens::Element => {    //We just need to go back one step without pushing anything to output yet if we have an element...
                                    tree = parser_structs::TreeNode::back_to_parent(&tree);
                                    token_in_tree = tree.borrow().value.clone();
                                },
                                _ => ()
                            }
                        }
                        line += 1;  //Increment the line counter.
                        } else {    //Alternatively, something other than a tab is there. So we must check if the tree contains something.
                            if tree_contains {  //If it does, we need to determine what we are dealing with. We have four options...
                                match token_in_tree {
                                    tokens::Tokens::Assignment => { //A type declaration which does not have any elements in its definition. We need to get to the parent of the assignment symbol.
                                        tree = parser_structs::TreeNode::back_to_parent(&tree);                                                                            //Move the tree variable namespace back to the type keyword which is the root of the type declaration statement.
                                        if tree.borrow().value.clone() != tokens::Tokens::TypeKeyword {  //Check that we have reached the type keyword. If not, throw an error.
                                            error_locator(line + 1, i);
                                            return Err("Invalid syntax (Parser): Found assignment symbol nested under a token other than the type keyword.")
                                        }
                                        output.push(parser_structs::IrElements::TypeDeclaration(Rc::clone(&tree)));    //Push the declaration statement to the output vector.
                                        (tree, tree_contains, token_in_tree) = tree_reset();                                                         //Replace the tree with a null value, set the tree_contains flag to false, and set the token_in_tree to a null token.
                                    },
                                    tokens::Tokens::E(_) => {   //Still a type declaration, but one which has elements in the definition, so the most recent node referenced in the variable is the element argument.
                                        tree = parser_structs::TreeNode::back_to_parent(&tree);                                                     //This first "back_to_parent" moves us to the assignment symbol.
                                        tree = parser_structs::TreeNode::back_to_parent(&tree);                                                     //This second one returns us to the desired type keyword.
                                        if tree.borrow().value.clone() != tokens::Tokens::TypeKeyword {  //Check that we have reached the type keyword. If not, throw an error.
                                            error_locator(line + 1, i);
                                            return Err("Invalid syntax (Parser): Found element argument nested under a token other than the type keyword. (Type keyword not two parents away)")
                                        }
                                        output.push(parser_structs::IrElements::TypeDeclaration(Rc::clone(&tree)));    //Push the declaration statement to the output vector.
                                        (tree, tree_contains, token_in_tree) = tree_reset();                                                         //Replace the tree with a null value, set the tree_contains flag to false, and set the token_in_tree to a null token.
                                    },
                                    tokens::Tokens::ContentWithFormatting(_) | tokens::Tokens::TypeInstance(_) => {   //Two situations that are handled similarly, either leading to raw text expression or a type expression. (These are just formatting differences.)
                                        if line_scope > last_line_scope { //If the current line's scope is greater than the previous one, do nothing. We are indented and therefore still inside the same expression.
                                        } else {
                                            let mut switch = tree.borrow().parent.as_ref().is_some();   //Otherwise, check if the node has a parent.
                                            while switch {
                                                tree = parser_structs::TreeNode::back_to_parent(&tree);             //Go back to the node's parent as long as there is a parent.
                                                switch = tree.borrow().parent.as_ref().is_some();
                                                token_in_tree = tree.borrow().value.clone();
                                            }
                                            match token_in_tree {   //Since there is a possibility that we've switched the tree, check what the parent node we are dealing with is.
                                                tokens::Tokens::ContentWithFormatting(_) => {   //Raw content, in which the content is the root of the tree (likely not a full tree in this case).
                                                    output.push(parser_structs::IrElements::RawText(Rc::clone(&tree))); //Now that there is no longer a parent node, we can push to the output vector.
                                                    (tree, tree_contains, token_in_tree) = tree_reset();
                                                },
                                                tokens::Tokens::TypeInstance(_) => {
                                                    output.push(parser_structs::IrElements::TypeExpression(Rc::clone(&tree)));
                                                    (tree, tree_contains, token_in_tree) = tree_reset();
                                                },
                                                _ => {  //Otherwise return an error.
                                                    error_locator(line, i);
                                                    return Err("Invalid Syntax (Parser): Invalid tree structure... root node is neither formatting nor a type expression.")
                                                }
                                            }
                                        }
                                    },
                                    tokens::Tokens::Element => {    //We just need to go back one step without pushing anything to output yet if we have an element...
                                        tree = parser_structs::TreeNode::back_to_parent(&tree);
                                        token_in_tree = tree.borrow().value.clone();
                                    },
                                    other => {  //Otherwise return an error.
                                        error_locator(line, other);
                                        return Err("Invalid syntax (Parser): The above token appears to have been used out of place. The parser is attempting to place it as the root of an expression's tree.")
                                    }
                                }
                            }
                            line += 1;  //Increment the line counter.
                        }
                    previous = i;
                },
                tokens::Tokens::Tab => {    //Tabs are used for spawning the "element" token that elements of collections are nested under.
                    if i == tokens::Tokens::Tab {   //Check if the next element is a tab. If it is, only increment the scope counter.
                        line_scope += 1;
                        previous = i;   //Before advancing the loop, remember to move the current i into previous.
                        continue
                    } else if tree_contains {   //Otherwise, check if the tree is empty. If not, increment the scope counter and then do something else!
                        //Either the tree is holding a type instance, or it is holding content with formatting.
                        if let tokens::Tokens::TypeInstance(_) = token_in_tree {  //Only spawn the element token for the type instances. Content with formatting will hold type instances directly under themselves.
                            tree = parser_structs::TreeNode::add_and_set(tokens::Tokens::Element, &tree);   //Spawn an element and make it the node that is being held in the tree variable.
                            token_in_tree = tokens::Tokens::Element;                                                                     //Update the token_in_tree.
                        } 
                    }
                    line_scope += 1;
                    previous = i;   //Before advancing the loop, remember to move the current i into previous.
                },
                //Type Declaration Statements
                tokens::Tokens::TypeKeyword => {    //Type Keyword. It will become the root of a tree for a type declaration statement.
                    if tree_contains {  //If a tree already exists, something is wrong. Type keywords don't go in other expressions.
                        error_locator(line, previous.clone());
                        return Err("Invalid Syntax (Parser): Type keyword was found in another expression or statement. It cannot be here.")
                    } else {    //Otherwise start a new tree with the type keyword as the root.
                    (tree, tree_contains, token_in_tree) = tree_fill(&previous.clone());    //Update tree_contains and token_in_tree appropriately.
                    }
                    previous = i;   //Before advancing the loop, remember to move the current i into previous.
                },
                tokens::Tokens::TypeName(s) => {    //Name of a type which is being declared. This will be the first child of a type keyword.
                    if tree_contains {  //Check that a tree exists.
                        if let tokens::Tokens::TypeKeyword = token_in_tree {   //Ensure that the value in the tree is a type keyword.
                            parser_structs::TreeNode::add(tokens::Tokens::TypeName(s.clone()), &tree);    //Add the type name to the tree.
                            previous = i;   //Before advancing the loop, remember to move the current i into previous.
                        } else {    //Return an error otherwise.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): Type name was not nested directly under a type keyword.")
                        }
                    } else {  //Return if no tree exists.
                        error_locator(line, previous.clone());
                        return Err("Invalid Syntax (Parser): Type name was not found in a type declaration.")
                    }
                },
                tokens::Tokens::Assignment => { //The assignment symbol. Should be the second child of a type keyword.
                    if tree_contains {  //Check that a tree exists.
                        if let tokens::Tokens::TypeKeyword = token_in_tree {    //Ensure that the tree's current node is a type keyword.
                            tree = parser_structs::TreeNode::add_and_set(tokens::Tokens::Assignment, &tree);    //Add the assignment symbol as a child of the type keyword, then shift the tree to hold the assignment symbol's node.
                            token_in_tree = tokens::Tokens::Assignment;     //Update the token_in_tree value to show that an assignment symbol is now the node at the head.
                            previous = i;   //Before advancing the loop, remember to move the current i into previous.
                        } else {    //Return an error otherwise.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): Assignment symbol was not nested directly under a type keyword.")
                        }
                    } else {  //Return if no tree exists.
                        error_locator(line, previous.clone());
                        return Err("Invalid Syntax (Parser): Assignment symbol was placed outside of a type declaration.")
                    }
                },
                tokens::Tokens::C(s) => {   //Content argument, of a type declaration statement.
                    if tree_contains {  //First confirm that a tree exists.
                        if let tokens::Tokens::Assignment | tokens::Tokens::E(_) = token_in_tree {  //A "c" can follow the assignment symbol or the element "e" argument
                        parser_structs::TreeNode::add(tokens::Tokens::C(s.clone()), &tree); //Add the token to the tree.
                            previous = i;   //Before advancing the loop, remember to move the current i into previous.
                        } else {    //Return an error otherwise.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): Content argument nested under wrong token. (Nested on something other than assignment symbol or element argument.")
                        }
                    } else {    //Return an error if no tree is found.
                        error_locator(line, previous.clone());
                        return Err("Invalid Syntax (Parser): Content argument placed outside of a type declaration.")
                    }
                },
                tokens::Tokens::E(s) => {   //Element argument of a type declaration statement.
                    if tree_contains {  //First confirm that a tree exists.
                        if let tokens::Tokens::Assignment = token_in_tree { //An "e" can only follow the assignment symbol.
                            tree = parser_structs::TreeNode::add_and_set(tokens::Tokens::E(s.clone()), &tree);  //Add the token to the tree and make it the node stored in the namespace.
                            token_in_tree = tokens::Tokens::E(s.clone());
                            previous = i;   //Before advancing the loop, remember to move the current i into previous.
                        } else {    //Return error if nested under another token.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): Element argument nested under wrong token. Can only be placed under an assignment symbol.")
                        }
                    } else {    //Return error if no tree found.
                        error_locator(line, previous.clone());
                        return Err("Invalid Syntax (Parser): Element argument placed outside of a type declaration.")
                    }
                },
                tokens::Tokens::Any =>  {   //Any argument of a type declaration statement.
                    if tree_contains {  //First confirm that a tree exists.
                        if let tokens::Tokens::Assignment | tokens::Tokens::E(_) = token_in_tree {  //Any can be placed after either the assignment symbol of the element argument.
                            parser_structs::TreeNode::add(tokens::Tokens::Any, &tree);  //Add the token to the tree.
                            previous = i;   //Move current i into previous.
                        } else {    //Return error if found nested under the wrong token.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): \"Any\" argument nested under wrong token. Can only be under either the assignment symbol or element argument.")
                        }
                    } else {    //Return error if no tree found.
                        error_locator(line, previous.clone());
                        return Err("Invalid Syntax (Parser): \"Any\" argument placed outside of a type declaration.")
                    }
                },
                tokens::Tokens::TypeAsDeclarationParameter(s) => {  //An argument of a type declaration statement that allows for type expressions to be nested in on another.
                    if tree_contains {  //First confirm that a tree exists.
                        if let tokens::Tokens::Assignment | tokens::Tokens::E(_) = token_in_tree {  //Can be placed after either the assignment symbol of the element argument.
                            parser_structs::TreeNode::add(tokens::Tokens::TypeAsDeclarationParameter(s.clone()), &tree);      //Add the token to the tree.
                            previous = i;   //Move current i into previous.
                        } else {    //Return error if found nested under the wrong token.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): Nested type argument nested under wrong token. Can only be under either the assignment symbol or element argument.")
                        }
                    } else {    //Return error if no tree found.
                        error_locator(line, previous.clone());
                        return Err("Invalid Syntax (Parser): Nested type argument placed outside of a type declaration.")
                    }
                },
                //Type Instances
                tokens::Tokens::TypeInstance(s) => {    //Instantiating a type.
                    if tree_contains {  //Check if a tree exists.
                        if let tokens::Tokens::Element | tokens::Tokens::ContentWithFormatting(_) = token_in_tree { //We can either place this under an element or contentWithFormatting token.
                            tree = parser_structs::TreeNode::add_and_set(tokens::Tokens::TypeInstance(s.clone()), &tree);   //Add and shift the tree.
                            token_in_tree = tokens::Tokens::TypeInstance(s.clone());    //Update the token_in_tree.
                        } else if let tokens::Tokens::TypeInstance(_) = token_in_tree { //Specific error message for nesting directly under another type.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): The name of the type does not need to be included in the same line as the parent type expression. The parser will automatically pattern match nested type expressions in line (e.g. anything before the elements of the colleciton).")
                        } else if let tokens::Tokens::TypeKeyword | tokens::Tokens::Assignment | tokens::Tokens::E(_) = token_in_tree { //Specific error message for nesting under a type declaration.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): Type expressions cannot be nested under type declarations.")
                        } else {    //Generic error message.
                            error_locator(line, previous.clone());
                            return Err("Invalid Syntax (Parser): Attempted to generate a type expression somewhere that one can't be placed.")
                        }
                    } else {    //Otherwise, create one.
                        (tree, tree_contains, token_in_tree) = tree_fill(&tokens::Tokens::TypeInstance(s.clone())); //Create a new tree, updating the relevant variables.
                    }
                    previous = i;   //Increment the line counter.
                },
                tokens::Tokens::Content(s) => {
                    match i {   //Check what the next token is. We need to know if it is more content that's going to get thrown in the same contentformatting block or a separator or other token, in which case we would be done immediately and package this and anything else in the content stack into a contentformatting block.
                        tokens::Tokens::CodeBlockOpen | tokens::Tokens::MathBlockOpen => {  //If it is open code block, open math block, do nothing, and push to stack.
                            formatting_stack.push(previous.clone());
                        },
                        tokens::Tokens::CodeBlockClose | tokens::Tokens::MathBlockClose => {    //If it is a close code block or math block, we would repackage this into the appropriate block and push to stack.
                            if let tokens::Tokens::CodeBlockClose = i {
                                formatting_stack.push(tokens::Tokens::CodeBlock(s.clone()));
                            } else {
                                formatting_stack.push(tokens::Tokens::MathBlock(s.clone()));
                            }
                        },
                        _ => {  //Anything else leads to previous being immediately placed in the wrapping content formatting block.
                            formatting_stack.push(previous.clone());
                            if tree_contains {  //We then check what we need to do based on whether or not there is a tree.
                                parser_structs::TreeNode::add(tokens::Tokens::ContentWithFormatting(formatting_stack.clone()), &tree);  //If there is, add to it.
                                formatting_stack = Vec::new();  //Reset the formatting_stack vector.
                            } else {    //If there is no tree, make one.
                                (tree, tree_contains, token_in_tree) = tree_fill(&tokens::Tokens::ContentWithFormatting(formatting_stack.clone()));
                                formatting_stack = Vec::new();  //Reset the formatting_stack vector.
                            }
                        }
                    }
                    previous = i;   //Reassign previous.
                },
                tokens::Tokens::CodeBlockClose | tokens::Tokens::MathBlockClose => {    //These would be skipped but must have the following token checked since it is possible that a separator follows them, requiring the formatting stack to be packaged.
                    if let tokens::Tokens::Content(_) | tokens::Tokens::CodeBlockOpen | tokens::Tokens::MathBlockOpen = i { }   //Do nothing in these cases.
                    else {  //Wrap in the contentwithformatting and push to the tree, or create a new tree.
                        if tree_contains {  //Add to an existing tree.
                            parser_structs::TreeNode::add(tokens::Tokens::ContentWithFormatting(formatting_stack.clone()), &tree);
                            formatting_stack = Vec::new();
                        } else {    //Create a new tree.
                            (tree, tree_contains, token_in_tree) = tree_fill(&tokens::Tokens::ContentWithFormatting(formatting_stack.clone()));
                            formatting_stack = Vec::new();
                        }
                    }
                    previous = i;      //Reassign previous.
                },
                //Tokens that can be ignored by the parser.
                tokens::Tokens::Separator(_) | tokens::Tokens::UseKeyword | tokens::Tokens::Filename(_) | tokens::Tokens::CommentLine | tokens::Tokens::CommentOpen
                | tokens::Tokens::CommentContents(_) | tokens::Tokens::CodeBlockOpen | tokens::Tokens::MathBlockOpen => {
                    previous = i;   //Do this and nothing else.
                },
                //Illegal Tokens. These are made by the parser but should not be found by the parser when parsing the lexer's output. The lexer cannot make these. Throw errors for all of them.
                tokens::Tokens::Element | tokens::Tokens::ContentWithFormatting(_) | tokens::Tokens::CodeBlock(_) | tokens::Tokens::MathBlock(_) => {
                    error_locator(line, previous.clone());
                    return Err("Parser error: Parser found tokens which cannot be created by the lexer.")
                }
            }
        }
        Ok(output)
    }
}

//Module containing the interpreter. The declaration statements' trees are converted into abstract types, and expressions' trees are then pattern matched and validated against those before the creation of the actual objects.
pub mod interpreter {
    use std::{rc::Rc, cell::RefCell};
    use crate::core::{tokens, parser_structs, interpreter_structs};

    pub fn interpreter(input: Vec<parser_structs::IrElements>) -> Vec<interpreter_structs::DiazoObject> {

        let mut output: Vec<interpreter_structs::DiazoObject> = Vec::new();
        let mut tree: Rc<RefCell<parser_structs::TreeNode>>;    //The tree being analyzed by the interpreter.

        for i in input {
            //Unwrap the tree stored in the IrElement.
            tree = i.unwrap().unwrap();
        }
        output
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tree_builder() {
        use crate::core::tokens;
        use std::rc::Rc;
        use std::cell::RefCell;
        use crate::core::parser_structs::TreeNode;

        let v = vec![
            tokens::Tokens::Any,
            tokens::Tokens::Assignment,
            tokens::Tokens::Any,
            tokens::Tokens::CommentOpen,
            tokens::Tokens::CodeBlockOpen
        ];

        let mut tree = Rc::new(RefCell::new(TreeNode::new(tokens::Tokens::TypeKeyword)));
    
        for i in v {
            TreeNode::add(i, &tree)
        }

        println!("{}", TreeNode::print(&*tree.borrow()));
        tree = TreeNode::add_and_set(tokens::Tokens::Element, &tree);
        println!("{}", TreeNode::print(&*tree.borrow()));
        tree = TreeNode::back_to_parent(&tree);
        println!("{}", TreeNode::print(&*tree.borrow()));
    }

}