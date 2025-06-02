use widow::parser;

fn main() {
    let source = r#"
        # Comprehensive test of all grammar features
        
        # Variable declarations with different types
        let x: i32 = 5 + 3 * (2 - 1);
        let y: f64 = (10.5 + 3.7) / 2.0;
        let isValid: bool = true;
        let name: String = "Hello World";
        let count = 42;
        let flag = false;
        
        # Constants with various types
        const PI: f64 = 3.14159;
        const MAX_SIZE: i32 = 1000;
        const DEBUG: bool = true;
        const MESSAGE: String = "System Ready";
        
        # Simple function with single return
        func add(a: i32, b: i32) -> i32 {
            let temp: i32 = a + b;
            ret temp;
        }
        
        # Function with multiple returns
        func calculate(a: i32, b: i32) -> (i32, bool) {
            let result: i32 = a * b + 10;
            let isPositive: bool = result > 0;
            ret result, isPositive;
        }
        
        # Function with no return type
        func printMessage(msg: String) {
            ret;
        }
        
        # Struct definition
        struct Person {
            name: String,
            age: i32,
            active: bool
        }
        
        # Implementation block
        impl Person {
            func getName(self: Person) -> String {
                ret self.name;
            }
            
            func setAge(self: Person, newAge: i32) {
                self.age = newAge;
            }
        }
        
        # Arrays with different expressions
        let numbers: [i32] = [1, 2, 3, (x + y), add(5, 3)];
        let names: [String] = ["Alice", "Bob", "Charlie"];
        let flags: [bool] = [true, false, (x > 0)];
        
        # Maps with various key-value types
        let config: {String: i32} = {"width": 800, "height": 600, "depth": (x * 2)};
        let userData: {String: String} = {"name": "John", "city": "NYC"};
        
        # Function calls with complex expressions
        let result1: i32 = add(x + 5, y * 2);
        let result2: i32 = add(add(1, 2), add(3, 4));
        
        # Control flow - if statements
        if x > 0 {
            let positive: bool = true;
        } elif x < 0 {
            let negative: bool = true;
        } else {
            let zero: bool = true;
        }
        
        # For loops
        for i in numbers {
            let processed: i32 = i * 2;
        }
        
        for item in 1..10 {
            let squared: i32 = item * item;
        }
        
        # Switch statements
        switch x {
            case 0, 1:
                let small: bool = true;
            case 5:
                let medium: bool = true;
            default:
                let large: bool = true;
        }
        
        # Complex expressions with all operators
        let complexResult: i32 = ((x + y) * 2 - 5) / (add(3, 4) + 1);
        let comparison: bool = (x >= y) && (result1 != result2);
        
        # Nested function calls and expressions
        let finalResult: i32 = add(calculate(x, y).0, add(x * 2, y + 3));
        
        # Return statements in main scope
        ret finalResult;
    "#;

    match parser::parse_source(source) {
        Ok(_) => println!("Parse successful!"),
        Err(e) => println!("Parse error: {:#?}", e),
    }
}
