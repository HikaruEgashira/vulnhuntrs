use tree_sitter::{Parser, Language, Query, QueryCursor};
use std::path::PathBuf;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub source: String,
}

pub fn get_language(file_ext: &str) -> Option<Language> {
    match file_ext {
        "py" => Some(tree_sitter_python::language()),
        "js" => Some(tree_sitter_javascript::language()),
        "ts" => Some(tree_sitter_typescript::language_typescript()),
        "java" => Some(tree_sitter_java::language()),
        _ => None,
    }
}

fn get_query(file_ext: &str) -> Option<Query> {
    let language = get_language(file_ext)?;
    let query_source = match file_ext {
        "py" => include_str!("queries/python.scm"),
        "js" => include_str!("queries/javascript.scm"),
        "ts" => include_str!("queries/typescript.scm"),
        "java" => include_str!("queries/java.scm"),
        _ => return None,
    };
    println!("Loading query for {}: {}", file_ext, query_source);
    match Query::new(language, query_source) {
        Ok(query) => {
            println!("Query loaded successfully with {} patterns", query.pattern_count());
            Some(query)
        }
        Err(e) => {
            println!("Failed to load query: {}", e);
            None
        }
    }
}

pub fn enumerate_definitions(
    files: Vec<PathBuf>,
) -> Result<Vec<(PathBuf, Vec<Definition>)>, Box<dyn Error>> {
    let mut results = Vec::new();
    let mut parser = Parser::new();

    for file_path in files {
        println!("Processing file: {:?}", file_path);
        let content = std::fs::read_to_string(&file_path)?;
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if let Some(language) = get_language(ext) {
            println!("Language found for extension {}", ext);
            parser.set_language(language)?;
            let tree = parser.parse(&content, None).unwrap();
            println!("Parsed tree: {}", tree.root_node().to_sexp());

            let mut definitions = Vec::new();
            if let Some(query) = get_query(ext) {
                println!("Query loaded for {}", ext);
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

                for match_ in matches {
                    println!("Found match: pattern {}", match_.pattern_index);
                    let mut captures_by_pattern = std::collections::HashMap::new();
                    for capture in match_.captures {
                        let capture_name = &query.capture_names()[capture.index as usize];
                        println!("Capture: {} -> {}", capture_name, capture.node.kind());
                        captures_by_pattern.insert(capture_name.as_str(), capture.node);
                    }

                    match ext {
                        "py" => {
                            // Handle Python functions and methods
                            if let Some(name_node) = captures_by_pattern.get("function.name") {
                                let name = content[name_node.byte_range()].to_string();
                                let mut current = name_node.parent();
                                let mut is_method = false;
                                
                                // Check if this function is inside a class
                                while let Some(node) = current {
                                    if node.kind() == "class_definition" {
                                        is_method = true;
                                        break;
                                    }
                                    current = node.parent();
                                }
                                
                                if !is_method {
                                    println!("Found Python function: {}", name);
                                    definitions.push(Definition {
                                        name: name.clone(),
                                        start_byte: name_node.start_byte(),
                                        end_byte: name_node.end_byte(),
                                        source: name,
                                    });
                                }
                            }

                            // Handle Python classes and methods
                            if let Some(name_node) = captures_by_pattern.get("class.name") {
                                let name = content[name_node.byte_range()].to_string();
                                println!("Found Python class: {}", name);
                                definitions.push(Definition {
                                    name: name.clone(),
                                    start_byte: name_node.start_byte(),
                                    end_byte: name_node.end_byte(),
                                    source: name,
                                });

                                // Find methods in this class
                                if let Some(class_node) = name_node.parent() {
                                    if let Some(body_node) = class_node.child_by_field_name("body") {
                                        for child in body_node.named_children(&mut body_node.walk()) {
                                            if child.kind() == "function_definition" {
                                                if let Some(method_name_node) = child.child_by_field_name("name") {
                                                    let method_name = content[method_name_node.byte_range()].to_string();
                                                    println!("Found Python method: {}", method_name);
                                                    definitions.push(Definition {
                                                        name: method_name.clone(),
                                                        start_byte: method_name_node.start_byte(),
                                                        end_byte: method_name_node.end_byte(),
                                                        source: method_name,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "js" | "ts" => {
                            // Handle JavaScript/TypeScript functions
                            if let Some(name_node) = captures_by_pattern.get("function.name") {
                                let name = content[name_node.byte_range()].to_string();
                                println!("Found JS/TS function: {}", name);
                                definitions.push(Definition {
                                    name: name.clone(),
                                    start_byte: name_node.start_byte(),
                                    end_byte: name_node.end_byte(),
                                    source: name,
                                });
                            }

                            // Handle JavaScript/TypeScript classes and methods
                            if let Some(name_node) = captures_by_pattern.get("class.name") {
                                let name = content[name_node.byte_range()].to_string();
                                println!("Found JS/TS class: {}", name);
                                definitions.push(Definition {
                                    name: name.clone(),
                                    start_byte: name_node.start_byte(),
                                    end_byte: name_node.end_byte(),
                                    source: name,
                                });
                            }

                            if let Some(body_node) = captures_by_pattern.get("class.body") {
                                for child in body_node.named_children(&mut body_node.walk()) {
                                    if child.kind() == "method_definition" {
                                        if let Some(name_node) = child.child_by_field_name("name") {
                                            let name = content[name_node.byte_range()].to_string();
                                            if name != "constructor" {
                                                println!("Found JS/TS method: {}", name);
                                                definitions.push(Definition {
                                                    name: name.clone(),
                                                    start_byte: name_node.start_byte(),
                                                    end_byte: name_node.end_byte(),
                                                    source: name,
                                                });
                                            }
                                        }
                                    }
                                }
                            }

                            // Handle TypeScript interfaces and types
                            if ext == "ts" {
                                if let Some(name_node) = captures_by_pattern.get("interface.name") {
                                    let name = content[name_node.byte_range()].to_string();
                                    println!("Found TS interface: {}", name);
                                    definitions.push(Definition {
                                        name: name.clone(),
                                        start_byte: name_node.start_byte(),
                                        end_byte: name_node.end_byte(),
                                        source: name,
                                    });
                                }

                                if let Some(name_node) = captures_by_pattern.get("type.name") {
                                    let name = content[name_node.byte_range()].to_string();
                                    println!("Found TS type: {}", name);
                                    definitions.push(Definition {
                                        name: name.clone(),
                                        start_byte: name_node.start_byte(),
                                        end_byte: name_node.end_byte(),
                                        source: name,
                                    });
                                }
                            }

                            // Handle variables
                            if let Some(name_node) = captures_by_pattern.get("variable.name") {
                                let name = content[name_node.byte_range()].to_string();
                                println!("Found JS/TS variable: {}", name);
                                definitions.push(Definition {
                                    name: name.clone(),
                                    start_byte: name_node.start_byte(),
                                    end_byte: name_node.end_byte(),
                                    source: name,
                                });
                            }
                        }
                        "java" => {
                            // Track what we've already captured to avoid duplicates
                            let mut captured_names = std::collections::HashSet::new();

                            // Handle Java classes and their methods
                            if let Some(name_node) = captures_by_pattern.get("class.name") {
                                let name = content[name_node.byte_range()].to_string();
                                println!("Found Java class: {}", name);
                                captured_names.insert(name.clone());
                                definitions.push(Definition {
                                    name: name.clone(),
                                    start_byte: name_node.start_byte(),
                                    end_byte: name_node.end_byte(),
                                    source: name,
                                });

                                // Find methods in this class
                                if let Some(class_node) = name_node.parent() {
                                    if let Some(body_node) = class_node.child_by_field_name("body") {
                                        for child in body_node.named_children(&mut body_node.walk()) {
                                            match child.kind() {
                                                "method_declaration" => {
                                                    if let Some(method_name_node) = child.child_by_field_name("name") {
                                                        let method_name = content[method_name_node.byte_range()].to_string();
                                                        if method_name != name { // Skip constructors
                                                            println!("Found Java method: {}", method_name);
                                                            captured_names.insert(method_name.clone());
                                                            definitions.push(Definition {
                                                                name: method_name.clone(),
                                                                start_byte: method_name_node.start_byte(),
                                                                end_byte: method_name_node.end_byte(),
                                                                source: method_name,
                                                            });
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }

                            // Handle interfaces
                            if let Some(name_node) = captures_by_pattern.get("interface.name") {
                                let name = content[name_node.byte_range()].to_string();
                                if !captured_names.contains(&name) {
                                    println!("Found Java interface: {}", name);
                                    captured_names.insert(name.clone());
                                    definitions.push(Definition {
                                        name: name.clone(),
                                        start_byte: name_node.start_byte(),
                                        end_byte: name_node.end_byte(),
                                        source: name,
                                    });
                                }
                            }

                            // Handle enums
                            if let Some(name_node) = captures_by_pattern.get("enum.name") {
                                let name = content[name_node.byte_range()].to_string();
                                if !captured_names.contains(&name) {
                                    println!("Found Java enum: {}", name);
                                    captured_names.insert(name.clone());
                                    definitions.push(Definition {
                                        name: name.clone(),
                                        start_byte: name_node.start_byte(),
                                        end_byte: name_node.end_byte(),
                                        source: name,
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                println!("No query found for extension {}", ext);
            }

            println!("Found {} definitions", definitions.len());
            results.push((file_path, definitions));
        } else {
            println!("No language support for extension {}", ext);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn setup_test_file(dir: &std::path::Path, filename: &str, content: &str) -> std::io::Result<()> {
        fs::write(dir.join(filename), content)
    }

    #[test]
    fn test_get_language() {
        assert!(get_language("py").is_some());
        assert!(get_language("js").is_some());
        assert!(get_language("ts").is_some());
        assert!(get_language("java").is_some());
        assert!(get_language("unknown").is_none());
    }

    #[test]
    fn test_enumerate_definitions_python() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_file(
            temp_dir.path(),
            "test.py",
            r#"def test_function():
    return "Hello, World!"

class TestClass:
    def __init__(self):
        pass

    def method(self):
        return test_function()"#,
        )?;

        let files = vec![temp_dir.path().join("test.py")];
        let results = enumerate_definitions(files)?;

        assert_eq!(results.len(), 1);
        let (_, definitions) = &results[0];
        
        // Should find test_function, TestClass, __init__, and method
        assert_eq!(definitions.len(), 4);
        
        let names: Vec<_> = definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"test_function"));
        assert!(names.contains(&"TestClass"));
        assert!(names.contains(&"__init__"));
        assert!(names.contains(&"method"));

        Ok(())
    }

    #[test]
    fn test_enumerate_definitions_javascript() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_file(
            temp_dir.path(),
            "test.js",
            r#"function testFunction() {
    return true;
}

class TestClass {
    constructor() {}
    
    method() {
        return testFunction();
    }
}

const constVar = 42;
let letVar = "test";"#,
        )?;

        let files = vec![temp_dir.path().join("test.js")];
        let results = enumerate_definitions(files)?;

        assert_eq!(results.len(), 1);
        let (_, definitions) = &results[0];
        
        // Should find testFunction, TestClass, method, constVar, and letVar
        assert_eq!(definitions.len(), 5);
        
        let names: Vec<_> = definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"testFunction"));
        assert!(names.contains(&"TestClass"));
        assert!(names.contains(&"method"));
        assert!(names.contains(&"constVar"));
        assert!(names.contains(&"letVar"));

        Ok(())
    }

    #[test]
    fn test_enumerate_definitions_typescript() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_file(
            temp_dir.path(),
            "test.ts",
            r#"interface TestInterface {
    name: string;
    value: number;
}

class TestTypeScriptClass implements TestInterface {
    name: string;
    value: number;

    constructor(name: string, value: number) {
        this.name = name;
        this.value = value;
    }

    public getValue(): number {
        return this.value;
    }
}

function testTypeScriptFunction(param: TestInterface): string {
    return param.name;
}

const typeScriptConst: number = 42;
let typeScriptLet: string = "test";

type CustomType = {
    id: number;
    label: string;
};"#,
        )?;

        let files = vec![temp_dir.path().join("test.ts")];
        let results = enumerate_definitions(files)?;

        assert_eq!(results.len(), 1);
        let (_, definitions) = &results[0];
        
        // Should find TestInterface, TestTypeScriptClass, getValue, testTypeScriptFunction,
        // typeScriptConst, typeScriptLet, CustomType
        assert_eq!(definitions.len(), 7);
        
        let names: Vec<_> = definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"TestInterface"));
        assert!(names.contains(&"TestTypeScriptClass"));
        assert!(names.contains(&"getValue"));
        assert!(names.contains(&"testTypeScriptFunction"));
        assert!(names.contains(&"typeScriptConst"));
        assert!(names.contains(&"typeScriptLet"));
        assert!(names.contains(&"CustomType"));

        Ok(())
    }

    #[test]
    fn test_enumerate_definitions_java() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_file(
            temp_dir.path(),
            "test.java",
            r#"public class TestClass {
    private String field;

    public TestClass() {
        this.field = "test";
    }

    public void testMethod() {
        System.out.println(field);
    }

    interface TestInterface {
        void interfaceMethod();
    }

    enum TestEnum {
        ONE, TWO, THREE
    }
}"#,
        )?;

        let files = vec![temp_dir.path().join("test.java")];
        let results = enumerate_definitions(files)?;

        assert_eq!(results.len(), 1);
        let (_, definitions) = &results[0];
        
        // Should find TestClass, testMethod, TestInterface, TestEnum
        assert_eq!(definitions.len(), 4);
        
        let names: Vec<_> = definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"TestClass"));
        assert!(names.contains(&"testMethod"));
        assert!(names.contains(&"TestInterface"));
        assert!(names.contains(&"TestEnum"));

        Ok(())
    }
}
