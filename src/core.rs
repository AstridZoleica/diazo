//Module for storing a variety of important pieces of the language:
// A - Tokens for use by the lexer
// B - Data structures for use by the parser
// C - The data structure representation of diazo types. For use in the interpreter itself.

//A - Define tokens for the lexer to use
pub mod tokens {
    // Define tokens for operators.
    #[derive(PartialEq)]
    #[derive(Debug)]
    pub enum Tokens {
        //Different Types of Separators, no difference between forms. End the current scope and switch to content.
        Separator(String),
        //Functional whitespace, each tab is used to determine scope.
        Tab,
        Linebreak,
        //Keywords for working with multiple files.
        UseKeyword,
        Filename(String),
        //Comments, with option for commenting by line or for block comments
        CommentLine,
        CommentOpen,
        CommentContents(String),
        //Type declarations
        TypeKeyword,
        TypeName(String),   //The name of a type being declared
        Assignment,
        E(String),  //Element formatter for collections
        C(String),  //Content formatter for any type
        Any,    //Keyword to allow use of any type
        TypeAsDeclarationParameter(String), //Type used as a declaration parameter
        //Markup Content
        TypeInstance(String), //Types instantiated during markup
        Content(String), //Notes being marked up
        //Formatting blocks for including code or mathematical expressions, still content technically,
        CodeBlockOpen,
        CodeBlockClose,
        MathBlockOpen,
        MathBlockClose,
        //Hidden tokens that are created and used by the parser rather than the lexer.
        Element, //An "element" that places the elements of collections as children under itself for later pattern matching.
        ContentWithFormatting(Vec<Tokens>), //Store a vector of content and formatting blocks if formatting blocks are found.
        MathBlock(String), //MathBlock. The lexer uses code block symbols to flank a Content token. These will be replaced with a single MathBlock token containing the appropriate text in the parser. 
        CodeBlock(String), //Similar process to the above.
        Null    //Null token because Option<> syntax is annoying.
    }

    impl Tokens {
        pub fn print(&self) -> String {
            //String adder function to reduce boilerplate
            fn string_adder(text: &str, s: &String) -> String {
                    text.to_string() + s.as_str()
            }
            //Match each of the variants of a token with a printout
            match self {
                Tokens::Separator(s) => string_adder("Separator Token of form ", s),
                Tokens::Tab => "Tab Token".to_string(),
                Tokens::Linebreak => "Linebreak Token".to_string(),
                Tokens::UseKeyword => "Use Keyword Token".to_string(),
                Tokens::Filename(s) => string_adder("Filename Token containing filename: ", s),
                Tokens::CommentLine => "Comment Symbol Token".to_string(),
                Tokens::CommentOpen => "Open Comment Block Symbol Token".to_string(),
                Tokens::CommentContents(s) => string_adder("Comment Token containing the text: ", s),
                Tokens::TypeKeyword => "Type Keyword Token".to_string(),
                Tokens::TypeName(s) => string_adder("Type Name Token holding name: ", s),
                Tokens::Assignment => "Assignment Symbol Token".to_string(),
                Tokens::E(s) => string_adder("Element Formatter Token for declarations containing: ", s),
                Tokens::C(s) => string_adder("Content Formatter Token for declarations containing: ", s),
                Tokens::Any => "\"Any\" Keyword Token".to_string(),
                Tokens::TypeAsDeclarationParameter(s) => string_adder("Type Used as Declaration Parameter, type: ", s),
                Tokens::TypeInstance(s) => string_adder("Type Instance Token, type: ", s),
                Tokens::Content(s) => string_adder("Content Token containing the text: ", s),
                Tokens::CodeBlockOpen => "Open Code Block Symbol Token".to_string(),
                Tokens::CodeBlockClose => "Close Code Block Symbol Token".to_string(),
                Tokens::MathBlockOpen => "Open Math Block Symbol Token".to_string(),
                Tokens::MathBlockClose => "Close Math Block Symbol Token".to_string(),
                Tokens::Element => "Element Node Token".to_string(),
                Tokens::ContentWithFormatting(_v) => "Content and Formatting Container".to_string(),
                Tokens::MathBlock(s) => string_adder("Math block containing the text: ", s),
                Tokens::CodeBlock(s) => string_adder("Code block containing the text: ", s),
                Tokens::Null => "Null Token".to_string(),
            }
        }
    }

    impl Clone for Tokens {
        fn clone(&self) -> Self {
            match self {
                Tokens::Separator(s) => Tokens::Separator(s.clone()),
                Tokens::Tab => Tokens::Tab ,
                Tokens::Linebreak => Tokens::Linebreak,
                Tokens::UseKeyword => Tokens::UseKeyword,
                Tokens::Filename(s) => Tokens::Filename(s.clone()),
                Tokens::CommentLine => Tokens::CommentLine,
                Tokens::CommentOpen => Tokens::CommentOpen,
                Tokens::CommentContents(s) => Tokens::CommentContents(s.clone()),
                Tokens::TypeKeyword => Tokens::TypeKeyword,
                Tokens::TypeName(s) => Tokens::TypeName(s.clone()),
                Tokens::Assignment =>  Tokens::Assignment,
                Tokens::E(s) => Tokens::E(s.clone()),
                Tokens::C(s) => Tokens::C(s.clone()),
                Tokens::Any => Tokens::Any,
                Tokens::TypeAsDeclarationParameter(s) => Tokens::TypeAsDeclarationParameter(s.clone()),
                Tokens::TypeInstance(s) => Tokens::TypeInstance(s.clone()),
                Tokens::Content(s) => Tokens::Content(s.clone()),
                Tokens::CodeBlockOpen => Tokens::CodeBlockOpen,
                Tokens::CodeBlockClose => Tokens::CodeBlockClose,
                Tokens::MathBlockOpen => Tokens::MathBlockOpen,
                Tokens::MathBlockClose => Tokens::MathBlockClose,
                Tokens::Element => Tokens::Element,
                Tokens::ContentWithFormatting(v) => Tokens::ContentWithFormatting(v.clone()),
                Tokens::MathBlock(s) => Tokens::MathBlock(s.clone()),
                Tokens::CodeBlock(s) => Tokens::CodeBlock(s.clone()),
                Tokens::Null => Tokens::Null,
            }
        }
    }
    
    // Return a list of accepted separator variants
    pub fn separator_list() -> Vec<&'static str> {
        vec!["::", "->", ",,"]
    }
}

//B - Define some data structures used for building the intermediate representation.
pub mod parser_structs {
    use std::{rc::Rc, cell::RefCell};

//Define the data structure of the nodes of the Abstract Syntax Tree.
#[derive(Debug)]    //Try not to use this as much as possible.
    pub struct TreeNode {
        pub value: super::tokens::Tokens,                  //The Token held in this node.
        pub children: Vec<Rc<RefCell<TreeNode>>>,   //List of children.
        pub parent: Option<Rc<RefCell<TreeNode>>>   //Parent node, of it exists.
    }

    impl TreeNode {

        //Initialize A Tree Node
        pub fn new(token: super::tokens::Tokens) -> TreeNode {
            TreeNode { value: token, children: Vec::new(), parent: None }
        }

        //Add a new node to an existing node/subtree/tree
        pub fn add(token: super::tokens::Tokens, existing_tree: &Rc<RefCell<TreeNode>>) {
            let mut t = TreeNode::new(token);
            t.parent = Some(Rc::clone(existing_tree));
            existing_tree.borrow_mut().children.push(Rc::new(RefCell::new(t)));
        }

        //Add a new node to an existing node/subtree/tree and then set it as the current node
        pub fn add_and_set(token: super::tokens::Tokens, existing_tree: &Rc<RefCell<TreeNode>>) -> Rc<RefCell<TreeNode>> {
            TreeNode::add(token, existing_tree);
            Rc::clone(&existing_tree.borrow().children[existing_tree.borrow().children.len() - 1])
        }

        //Move back a node and use a reference to the parent node.
        pub fn back_to_parent(existing_tree: &Rc<RefCell<TreeNode>>) -> Rc<RefCell<TreeNode>> {
            Rc::clone(existing_tree.borrow_mut().parent.as_ref().unwrap())
        }

        //Print out the node's contents to a string. Debug doesn't work on these since it tries to print a reference to the child, which references the parent, which references the child... and so on.
        pub fn print(&self) -> String {
            //String adder function to reduce boilerplate
            fn string_adder(text: &str, s: &String) -> String { text.to_string() + s.as_str() }

            let mut output: String = String::from("");  //String that will be assembled and eventually returned.
            let queue: Vec<String> = vec![              //Vector that will be iterated over and fed into String.
                "AST Node Containing: ".to_string(),
                string_adder("\n\tToken: ", &self.value.print()),
                "\n\tChildren: \n\t".to_string(),
                match &self.children.len() {            //Check if there is anything in the children vector.
                    0 => "\tNo children.".to_string(),  //If not, say so.
                    _ => {                              //Otherwise, assemble a String from the children of the node.
                        let mut t = String::new();
                        for i in &self.children {
                            t = t + i.borrow().value.print().as_str() + "\n\t\t";
                        }
                        t
                    }
                },
                "\n\tParent: ".to_string(),
                match &self.parent {    //Check if the node has a parent, since that is an Option<T>.
                    Some(x) => x.borrow().value.print(),
                    None => "No parent.".to_string()
                }
            ];
            //Add together all the strings held in the vector to produce the output.
            for i in queue {
                output = string_adder(output.as_str(), &i)
            }
            output
        }

        //Read the tree using a preorder traversal method. Output a vector of the tokens contained inside the tree.
        pub fn preorder_read(&self) -> Vec<ReaderTuple> {
            let mut output: Vec<ReaderTuple> = Vec::new();  //Initialize the output vector.
            let mut children_found: bool = false;                   //Boolean to store wheether or not the node has children.
            let mut id_counter: usize = 0;                              //Id counter that will be assigned to each node's token.
            let mut child_counter: usize = 0;                        //Child counter that lets us loop until we have finished with all of the children.
            let mut child_max: usize = 0;                               //Maximum number of children.
            let mut parent_id: usize = 0;

            let mut node:Rc<RefCell<TreeNode>> = Rc::new(RefCell::new(TreeNode::new(super::tokens::Tokens::Null)));

            output.push(ReaderTuple(id_counter, None, self.value.clone()));

            if self.children.len() > 0 { children_found = true; }   //If the first node has children, set the bool to true.
            child_max = self.children.len() - 1;    //Set the max to the highest index in the vector, which is length - 1.
            while children_found && child_counter <= child_max {    //As long as we have children, and we have not yet reached the last child...
                id_counter += 1;    //Increment the id_counter.
                node = Rc::clone(&self.children[child_counter]);    //Make this child the current node.
                output.push(ReaderTuple(id_counter, Some(parent_id), node.borrow().value.clone())); //Push this node's information to the output stack.
                if node.borrow().children.len() > 0 {
                    children_found = true;  //Tell loop to keep looking at children.
                    parent_id = id_counter; //Change the parent_id to the current id_counter since we are about to start looking at those children.
                } else {
                    children_found = false;
                }   //Check if the node has children, and mutate the bool based on that.
            }
            output
        }

    }

    //A tuple struct for use in the impl block of TreeNodes.
    pub struct ReaderTuple(usize, Option<usize>, super::tokens::Tokens);

    //As the parser builds abstract syntax trees, it will need to sort through different tokens, some of which might not be necessary.
    //This enum's variants represent what is actually used later on in the interpreter, with other tokens being left behind or used to help build the tree.
    
    #[derive(Debug)]
    pub enum IrElements{
        TypeDeclaration(Rc<RefCell<TreeNode>>),
        TypeExpression(Rc<RefCell<TreeNode>>),
        RawText(Rc<RefCell<TreeNode>>)
    }

    impl IrElements{
        pub fn print(&self) -> String {
            match self {
                IrElements::RawText(n) | IrElements::TypeDeclaration(n) | IrElements::TypeExpression(n) => {
                    if n.borrow().children.len() > 0 {
                        for i in &n.borrow().children {
                            println!("{}", TreeNode::print(&i.borrow()));
                            if i.borrow().children.len() > 0 {
                                for j in &i.borrow().children {
                                    println!("{}", TreeNode::print(&j.borrow()));
                                }
                            }
                        }
                    }
                    TreeNode::print(&n.borrow())
                }
            }
        }
    
        pub fn unwrap(self) -> Option<Rc<RefCell<TreeNode>>> {
            if let IrElements::RawText(n) | IrElements::TypeDeclaration(n) | IrElements::TypeExpression(n) = self {
                Some(n)
            } else {
                None
            }
        }
    }
}

//C - Define the data structure of diazo types.
pub mod interpreter_structs {
    
    pub struct AbstractDType {

    }

    pub struct DiazoObject {

    }
}