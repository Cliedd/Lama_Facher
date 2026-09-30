"""Generate the curated Forge curriculum. Requires PyYAML.

Run from any directory: python3 /path/to/generate_exercises.py
"""

from pathlib import Path
import re
import yaml

from curriculum_fr import TEXTES


ROOT = Path(__file__).resolve().parent / "exercises"
LEGACY = re.compile(r"(?:\d{3}_(?:basic|loop|oop|control|structs)_\d+|0[12]_(?:hello_world|variables))\.yaml")
GENERATED = re.compile(r"\d{2}_[a-z_]+\.yaml")
CHAPTERS = [
    "Premiers pas", "Décisions et répétitions", "Code et données réutilisables",
    "Cas concrets", "Textes et collections", "Concevoir du code réutilisable",
    "Transformations sûres", "Petits programmes",
]
CHAPTERS_EN = [
    "Getting started", "Decisions and loops", "Reusable code and data",
    "Concrete cases", "Text and collections", "Designing reusable code",
    "Safe transformations", "Small programs",
]


def j(body):
    return "public class Main {\n    public static void main(String[] args) {\n" + body + "\n    }\n}\n"


def r(body):
    return "fn main() {\n" + body + "\n}\n"


# Each item: slug, title, chapter number, lesson, task, tip, starter, output, graded hints.
JAVA = [
    ("hello", "Print your first message", 1,
     "A Java program starts in main. System.out.println writes one line.",
     "Print Hello, Forge! exactly, including punctuation.",
     "Keep the public class named Main because the file is Main.java.",
     j('        // TODO: Print the greeting.\n        System.out.println("Replace me");'),
     "Hello, Forge!", ["Use System.out.println with a string literal.", 'Put "Hello, Forge!" inside the parentheses.']),
    ("variables", "Add two scores", 1,
     "An int stores a whole number. Expressions can combine variables.",
     "Print the sum of first and second.",
     "Calculate from the named variables instead of typing the answer.",
     j("        int first = 12;\n        int second = 8;\n        // TODO: Print their sum.\n        System.out.println(first);"),
     "20", ["Use the + operator.", "Pass first + second to System.out.println."]),
    ("strings", "Build a greeting", 1,
     "The + operator joins strings; spaces and punctuation must be included.",
     "Print Hello, Ada! using the name variable.",
     "Join a prefix, the variable, and punctuation.",
     j('        String name = "Ada";\n        // TODO: Use name in the greeting.\n        System.out.println(name);'),
     "Hello, Ada!", ["Join strings with +.", 'Print "Hello, " + name + "!".']),
    ("condition", "Choose a ticket price", 2,
     "An if/else chooses one branch. The < operator compares numbers.",
     "Visitors younger than 18 pay 5; everyone else pays 10. Print the price for age 16.",
     "Try the boundary age 18 after solving.",
     j("        int age = 16;\n        int price = 0;\n        // TODO: Set price with if/else.\n        System.out.println(price);"),
     "5", ["Compare age with 18 using <.", "Assign 5 in the if branch and 10 in the else branch."]),
    ("loop", "Sum a short sequence", 2,
     "A for loop initializes a counter, checks a condition, then updates the counter.",
     "Add the numbers 1 through 5 and print the total.",
     "Check that your loop includes both endpoints.",
     j("        int total = 0;\n        // TODO: Add 1 through 5 with a for loop.\n        System.out.println(total);"),
     "15", ["Loop from i = 1 while i <= 5.", "Inside the loop, update total with total += i."]),
    ("method", "Write a reusable method", 2,
     "A static method can be called from main. return sends a result back.",
     "Complete doubleValue so that doubleValue(7) prints 14.",
     "Keep the signature; replace only the placeholder return value.",
     "public class Main {\n    static int doubleValue(int value) {\n        // TODO: Return twice value.\n        return 0;\n    }\n    public static void main(String[] args) {\n        System.out.println(doubleValue(7));\n    }\n}\n",
     "14", ["Use the value parameter.", "Return value * 2."]),
    ("array", "Find the largest number", 3,
     "An array groups values of one type; length gives its element count.",
     "Find the maximum in {4, 9, 2, 7}.",
     "Start maximum at the first element so negative inputs work too.",
     j("        int[] numbers = {4, 9, 2, 7};\n        int maximum = numbers[0];\n        // TODO: Visit each number and update maximum.\n        System.out.println(maximum);"),
     "9", ["Loop while i < numbers.length.", "If numbers[i] > maximum, assign numbers[i] to maximum."]),
    ("class", "Give an object a method", 3,
     "A class stores fields; an instance method can read those fields.",
     "Make Rectangle.area() return width times height.",
     "The object construction and method call are already present.",
     "public class Main {\n    public static void main(String[] args) {\n        Rectangle shape = new Rectangle(3, 4);\n        System.out.println(shape.area());\n    }\n}\nclass Rectangle {\n    private final int width, height;\n    Rectangle(int width, int height) { this.width = width; this.height = height; }\n    int area() {\n        // TODO: Compute area.\n        return 0;\n    }\n}\n",
     "12", ["Both fields are available in area().", "Return width * height."]),
    ("list", "Collect passing scores", 3,
     "ArrayList grows as elements are added; size() gives its length.",
     "Add only scores of at least 50 and print how many passed.",
     "Focus on filtering; the import and list are ready.",
     "import java.util.ArrayList;\npublic class Main {\n    public static void main(String[] args) {\n        int[] scores = {42, 80, 55, 31};\n        ArrayList<Integer> passing = new ArrayList<>();\n        // TODO: Add each passing score.\n        System.out.println(passing.size());\n    }\n}\n",
     "2", ["Loop over scores and compare each score with 50.", "Call passing.add(score) when score >= 50."]),
    ("map", "Count word occurrences", 4,
     "A HashMap associates keys with values; getOrDefault handles missing keys.",
     "Count each word and print the count for rust.",
     "Use the word as the key, not a separate counter per word.",
     "import java.util.HashMap;\npublic class Main {\n    public static void main(String[] args) {\n        String[] words = {\"java\", \"rust\", \"rust\", \"java\", \"rust\"};\n        HashMap<String, Integer> counts = new HashMap<>();\n        // TODO: Count every word.\n        System.out.println(counts.getOrDefault(\"rust\", 0));\n    }\n}\n",
     "3", ["Loop over words and read getOrDefault(word, 0).", "Use counts.put(word, counts.getOrDefault(word, 0) + 1)."]),
    ("exception", "Handle invalid input", 4,
     "Integer.parseInt converts text to int and can throw NumberFormatException.",
     "Catch the invalid input and print invalid without crashing.",
     "Catch the specific exception rather than every Exception.",
     j('        String input = "twelve";\n        // TODO: Parse in a try block and handle the error.\n        System.out.println("Replace me");'),
     "invalid", ["Try Integer.parseInt(input).", 'In catch (NumberFormatException e), print "invalid".']),
    ("project", "Summarize a shopping basket", 4,
     "A small program combines arrays, loops, and conditions.",
     "Sum prices {3, 7, 2}, subtract 2 when the total exceeds 10, and print the result.",
     "Calculate from the data; do not print a fixed answer.",
     j("        int[] prices = {3, 7, 2};\n        int total = 0;\n        // TODO: Sum prices and apply the discount.\n        System.out.println(total);"),
     "10", ["Loop through prices and add each to total.", "After the loop, if total > 10, subtract 2."]),
]

# Chapters 5–8 extend the original twelve exercises without changing their IDs.
JAVA.extend([
    ("substring", "Extract a word", 5,
     "substring(start, end) takes characters from start up to, but not including, end.",
     "Extract Forge from the label and print it.",
     "Index 0 is the first character; the comma occupies index 5.",
     j('        String label = "Hello, Forge!";\n        // TODO: Extract Forge with substring.\n        System.out.println(label);'),
     "Forge", ["Find the F and the character after e.", "Use label.substring(7, 12)."]),
    ("trim", "Clean user input", 5,
     "trim removes leading and trailing whitespace; toLowerCase normalizes capitalization.",
     "Print the cleaned, lowercase value of input.",
     "Chain the two String methods instead of replacing the input manually.",
     j('        String input = "  RuSt  ";\n        // TODO: Trim whitespace and lowercase the text.\n        System.out.println(input);'),
     "rust", ["Call trim() first.", "Print input.trim().toLowerCase()."]),
    ("split", "Count comma-separated items", 5,
     "split turns text into an array using a separator; length counts the parts.",
     "Print how many languages are in the line.",
     "The separator is a literal comma in this example.",
     j('        String line = "Java,Rust,Go";\n        // TODO: Split the line and count its parts.\n        System.out.println(0);'),
     "3", ['Call line.split(",") to get a String array.', 'Print line.split(",").length.']),
    ("set", "Keep unique visitors", 5,
     "A HashSet stores each distinct value once, regardless of duplicates.",
     "Add every visitor and print the number of unique names.",
     "A set's size changes only when a new value is added.",
     'import java.util.HashSet;\n' + j('        String[] visitors = {"Ada", "Lin", "Ada", "Mia"};\n        HashSet<String> unique = new HashSet<>();\n        // TODO: Add every visitor to unique.\n        System.out.println(unique.size());'),
     "3", ["Loop over visitors.", "Inside the loop, call unique.add(visitor)."]),
    ("sort", "Order the scores", 5,
     "Arrays.sort sorts an array in ascending order, changing the original array.",
     "Sort the scores and print the middle value.",
     "The middle index of a three-element array is 1.",
     'import java.util.Arrays;\n' + j('        int[] scores = {9, 2, 5};\n        // TODO: Sort scores.\n        System.out.println(scores[1]);'),
     "5", ["Arrays has a sort method.", "Call Arrays.sort(scores) before printing."]),
    ("overload", "Overload a method", 6,
     "Methods can share a name when their parameter lists differ.",
     "Complete the two-argument sum method so the program prints 9.",
     "The one-argument method already works; keep its behavior.",
     'public class Main {\n    static int sum(int value) { return value; }\n    static int sum(int left, int right) {\n        // TODO: Combine both arguments.\n        return 0;\n    }\n    public static void main(String[] args) {\n        System.out.println(sum(4, 5));\n    }\n}\n',
     "9", ["Use both parameters.", "Return left + right."]),
    ("constructor", "Initialize an account", 6,
     "A constructor gives a new object its initial state using its arguments.",
     "Store the initial balance and print it through balance().",
     "this.balance refers to the object's field.",
     'public class Main {\n    public static void main(String[] args) {\n        Account account = new Account(25);\n        System.out.println(account.balance());\n    }\n}\nclass Account {\n    private int balance;\n    Account(int initial) {\n        // TODO: Save initial in the field.\n    }\n    int balance() { return balance; }\n}\n',
     "25", ["Assign the constructor argument to a field.", "Use this.balance = initial;."]),
    ("inheritance", "Override a greeting", 6,
     "A subclass can override an inherited method; dynamic dispatch uses the object's implementation.",
     "Override message() in Friendly so it prints Hello, Ada!.",
     "Keep the same method signature as the parent.",
     'public class Main {\n    public static void main(String[] args) {\n        Greeting greeting = new Friendly();\n        System.out.println(greeting.message());\n    }\n}\nclass Greeting { String message() { return "Hello"; } }\nclass Friendly extends Greeting {\n    // TODO: Override message() to add Ada.\n}\n',
     "Hello, Ada!", ["Declare String message() in Friendly.", 'Return "Hello, Ada!" from the override.']),
    ("interface", "Implement a rule", 6,
     "An interface defines a contract; an implementing class supplies the method body.",
     "Make EvenRule.accept(8) print true.",
     "The modulo operator finds the remainder after division.",
     'public class Main {\n    public static void main(String[] args) {\n        Rule rule = new EvenRule();\n        System.out.println(rule.accept(8));\n    }\n}\ninterface Rule { boolean accept(int value); }\nclass EvenRule implements Rule {\n    public boolean accept(int value) {\n        // TODO: Accept even numbers only.\n        return false;\n    }\n}\n',
     "true", ["Even numbers leave remainder 0 when divided by 2.", "Return value % 2 == 0;."]),
    ("optional", "Supply a missing name", 7,
     "Optional represents a value that may be absent; orElse provides a fallback.",
     "Print Guest when the name is empty.",
     "Avoid calling get() on an empty Optional.",
     'import java.util.Optional;\n' + j('        Optional<String> name = Optional.empty();\n        // TODO: Print the fallback name.\n        System.out.println("Replace me");'),
     "Guest", ["Call orElse on name.", 'Print name.orElse("Guest").']),
    ("stream_filter", "Filter even numbers", 7,
     "A stream can filter elements and count those that satisfy a predicate.",
     "Count the even values in numbers and print the result.",
     "The stream is consumed by count().",
     'import java.util.Arrays;\n' + j('        int[] numbers = {1, 2, 3, 4, 6};\n        // TODO: Filter even numbers in the stream.\n        System.out.println(0);'),
     "3", ["Start with Arrays.stream(numbers).", "Use filter(n -> n % 2 == 0).count()."]),
    ("stream_map", "Transform each price", 7,
     "mapToInt transforms stream elements into integers; sum reduces them to one total.",
     "Add 1 to each price and print the resulting sum.",
     "Transform values before calculating the total.",
     'import java.util.Arrays;\n' + j('        int[] prices = {2, 4, 6};\n        // TODO: Map each price to price + 1, then sum.\n        System.out.println(0);'),
     "15", ["Start with Arrays.stream(prices).", "Use map(price -> price + 1).sum()."]),
    ("parse_list", "Parse a list of numbers", 7,
     "Integer.parseInt converts numeric strings; a loop can accumulate the results.",
     "Parse all three values and print their sum.",
     "Do not concatenate the strings.",
     j('        String[] values = {"4", "6", "8"};\n        int total = 0;\n        // TODO: Parse and add each value.\n        System.out.println(total);'),
     "18", ["Loop over values.", "Add Integer.parseInt(value) to total each time."]),
    ("frequency", "Find the most common fruit", 8,
     "A frequency map counts how many times each key occurs.",
     "Count the fruits and print the count for apple.",
     "Reuse getOrDefault for missing keys.",
     'import java.util.HashMap;\n' + j('        String[] fruits = {"apple", "pear", "apple", "plum", "apple"};\n        HashMap<String, Integer> counts = new HashMap<>();\n        // TODO: Count each fruit.\n        System.out.println(counts.getOrDefault("apple", 0));'),
     "3", ["Loop over fruits, using each fruit as a key.", "Use counts.put(fruit, counts.getOrDefault(fruit, 0) + 1)."]),
    ("average", "Compute an integer average", 8,
     "An integer average divides a sum by the count; integer division drops any fraction.",
     "Calculate and print the average of the scores.",
     "Derive the divisor from scores.length.",
     j('        int[] scores = {10, 20, 30};\n        int total = 0;\n        // TODO: Add all scores and divide by their count.\n        System.out.println(total);'),
     "20", ["Loop through scores to build total.", "Print total / scores.length."]),
    ("inventory", "Update an inventory", 8,
     "A map can represent current stock; getOrDefault makes first updates simple.",
     "Add the deliveries and print the stock of pens.",
     "Each delivery adds to the existing stock.",
     'import java.util.HashMap;\n' + j('        String[] items = {"pen", "book", "pen"};\n        int[] amounts = {2, 3, 4};\n        HashMap<String, Integer> stock = new HashMap<>();\n        // TODO: Add each amount to its item.\n        System.out.println(stock.getOrDefault("pen", 0));'),
     "6", ["Use the same index for items and amounts.", "Use stock.put(items[i], stock.getOrDefault(items[i], 0) + amounts[i])."]),
    ("gradebook", "Count passing students", 8,
     "A gradebook combines a map of names and scores with a conditional count.",
     "Print how many students scored at least 50.",
     "Iterate over values(), since the names do not affect passing.",
     'import java.util.Map;\n' + j('        Map<String, Integer> grades = Map.of("Ada", 80, "Lin", 49, "Mia", 50);\n        int passed = 0;\n        // TODO: Count scores of at least 50.\n        System.out.println(passed);'),
     "2", ["Loop through grades.values().", "Increment passed when score >= 50."]),
    ("report", "Build a compact report", 8,
     "A report often combines filtering, counting, and formatting in one output line.",
     "Count values above 10 and print high=<count>.",
     "Calculate the count from the data before formatting.",
     j('        int[] values = {4, 12, 15, 8, 20};\n        int high = 0;\n        // TODO: Count values above 10.\n        System.out.println("high=" + high);'),
     "high=3", ["Loop over values and test value > 10.", "Increment high for each matching value."]),
])

RUST = [
    ("hello", "Print your first message", 1,
     "Rust starts in main. println! is a macro that writes one line.",
     "Print Hello, Forge! exactly, including punctuation.",
     "The exclamation mark belongs to the macro name.",
     r('    // TODO: Print the greeting.\n    println!("Replace me");'),
     "Hello, Forge!", ["Call println! with a string literal.", 'Put "Hello, Forge!" inside the parentheses.']),
    ("variables", "Change a score", 1,
     "Bindings are immutable by default; mut allows changes.",
     "Start with score 5, increase it by 7, and print it.",
     "Update score rather than writing a new fixed answer.",
     r('    let mut score = 5;\n    // TODO: Increase score by 7.\n    println!("{score}");'),
     "12", ["A mutable binding can be assigned a new value.", "Use score += 7; before println!."]),
    ("strings", "Format a greeting", 1,
     "println! inserts values into a string through braces. String owns its text.",
     "Print Hello, Ada! using the name variable.",
     "Put text around the formatting placeholder.",
     r('    let name = String::from("Ada");\n    // TODO: Use name in the greeting.\n    println!("{name}");'),
     "Hello, Ada!", ["Keep {name} inside the format string.", 'Use println!("Hello, {name}!");']),
    ("condition", "Choose a ticket price", 2,
     "if is an expression: its branches can produce a value.",
     "Visitors younger than 18 pay 5; everyone else pays 10. Print the price for age 16.",
     "Both branches must have compatible types.",
     r('    let age = 16;\n    // TODO: Replace the placeholder with an if expression.\n    let price = 0;\n    println!("{price}");'),
     "5", ["Compare age < 18.", "Use let price = if age < 18 { 5 } else { 10 };"]),
    ("loop", "Sum a short sequence", 2,
     "The range 1..=5 includes both endpoints; for visits each value.",
     "Add numbers 1 through 5 and print the total.",
     "total needs mut because the loop updates it.",
     r('    let mut total = 0;\n    // TODO: Add values in 1..=5.\n    println!("{total}");'),
     "15", ["Use for value in 1..=5 { ... }.", "Inside the loop, write total += value;."]),
    ("function", "Write a reusable function", 2,
     "A function declares parameter and return types; its final expression can be the result.",
     "Complete double_value so that double_value(7) prints 14.",
     "A final expression has no semicolon.",
     'fn double_value(value: i32) -> i32 {\n    // TODO: Return twice value.\n    0\n}\nfn main() {\n    println!("{}", double_value(7));\n}\n',
     "14", ["Use the value parameter.", "Make value * 2 the final expression."]),
    ("borrowing", "Borrow without moving", 3,
     "Passing &String lends access without transferring ownership.",
     "Complete length_of so the program prints 5 and then hello on separate lines.",
     "The caller must still be able to use message afterward.",
     'fn length_of(text: &String) -> usize {\n    // TODO: Return the borrowed text length.\n    0\n}\nfn main() {\n    let message = String::from("hello");\n    println!("{}", length_of(&message));\n    println!("{message}");\n}\n',
     "5\nhello", ["String has a len() method.", "Return text.len() from length_of."]),
    ("vector", "Find the largest number", 3,
     "A Vec stores values of one type; iterating over &numbers borrows the vector.",
     "Find the maximum in [4, 9, 2, 7].",
     "Start maximum at the first element so negative values work too.",
     r('    let numbers = vec![4, 9, 2, 7];\n    let mut maximum = numbers[0];\n    // TODO: Visit each number and update maximum.\n    println!("{maximum}");'),
     "9", ["Use for &number in &numbers { ... }.", "If number > maximum, assign number to maximum."]),
    ("struct", "Give a struct a method", 3,
     "A struct groups fields; an impl block adds methods that read them through &self.",
     "Make Rectangle::area return width times height.",
     "The struct instance and method call are provided.",
     'struct Rectangle { width: i32, height: i32 }\nimpl Rectangle {\n    fn area(&self) -> i32 {\n        // TODO: Compute area.\n        0\n    }\n}\nfn main() {\n    let shape = Rectangle { width: 3, height: 4 };\n    println!("{}", shape.area());\n}\n',
     "12", ["Use self.width and self.height.", "Make self.width * self.height the final expression."]),
    ("enum", "Match every status", 4,
     "An enum names variants; match must cover every possible variant.",
     "Print ready for Status::Ready, and busy for Status::Busy.",
     "Pattern matching translates states into user-facing text.",
     'enum Status { Ready, Busy }\nfn main() {\n    let status = Status::Ready;\n    // TODO: Match status and print the matching text.\n    println!("unknown");\n}\n',
     "ready", ["Use match status with two arms.", 'Call println!("ready") in the Ready arm and println!("busy") in the Busy arm.']),
    ("result", "Handle a parse error", 4,
     "parse::<i32>() returns Result; match Ok(value) and Err(_).",
     "Parse invalid input and print invalid without panicking.",
     "Avoid unwrap() because the input is invalid.",
     r('    let input = "twelve";\n    // TODO: Match input.parse::<i32>() and handle both cases.\n    println!("Replace me");'),
     "invalid", ["Match on input.parse::<i32>().", 'Print the number in Ok(value), and "invalid" in Err(_).']),
    ("project", "Summarize a shopping basket", 4,
     "A small program combines vectors, iteration, and conditions.",
     "Sum prices [3, 7, 2], subtract 2 when the total exceeds 10, and print the result.",
     "Calculate from the data; do not print a fixed answer.",
     r('    let prices = vec![3, 7, 2];\n    let mut total = 0;\n    // TODO: Sum prices and apply the discount.\n    println!("{total}");'),
     "10", ["Iterate over &prices and add each price to total.", "After the loop, if total > 10, subtract 2."]),
]

RUST.extend([
    ("string_slice", "Borrow a word slice", 5,
     "A string slice &str borrows part of a string without copying it.",
     "Print Forge by slicing the message.",
     "ASCII text uses byte positions that match character positions here.",
     r('    let message = String::from("Hello, Forge!");\n    // TODO: Borrow just the word Forge.\n    println!("{message}");'),
     "Forge", ["Start at the F after the comma and space.", "Print &message[7..12]."]),
    ("mutable_borrow", "Update through a borrow", 5,
     "A mutable reference &mut T lets a function update a value owned by its caller.",
     "Complete add_bonus so the score becomes 15.",
     "Dereference the mutable reference to reach the integer.",
     'fn add_bonus(score: &mut i32) {\n    // TODO: Add 5 through the mutable reference.\n}\nfn main() {\n    let mut score = 10;\n    add_bonus(&mut score);\n    println!("{score}");\n}\n',
     "15", ["Use *score to access the value.", "Write *score += 5; inside add_bonus."]),
    ("split", "Count words", 5,
     "split_whitespace yields words separated by any run of whitespace.",
     "Count and print the words in the sentence.",
     "Iterator count() consumes the iterator.",
     r('    let sentence = "Rust makes systems fun";\n    // TODO: Count the whitespace-separated words.\n    println!("0");'),
     "4", ["Call sentence.split_whitespace().", "Print sentence.split_whitespace().count()."]),
    ("chars", "Count vowels", 5,
     "chars iterates Unicode scalar values; matches! can test several alternatives.",
     "Count lowercase vowels in banana and print the count.",
     "The input is lowercase, so uppercase cases are unnecessary.",
     r('    let word = "banana";\n    let mut count = 0;\n    // TODO: Count vowels by iterating over chars.\n    println!("{count}");'),
     "3", ["Use for ch in word.chars().", "Increment when matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u')."]),
    ("generic", "Write a generic first item", 6,
     "A generic function can work with different element types using a type parameter.",
     "Complete first so the first number prints as 7.",
     "Borrow the slice and return a borrowed element.",
     'fn first<T>(items: &[T]) -> &T {\n    // TODO: Return the first element.\n    &items[1]\n}\nfn main() {\n    let numbers = [7, 8];\n    println!("{}", first(&numbers));\n}\n',
     "7", ["Index zero is the first element.", "Return &items[0]."]),
    ("trait", "Implement a trait", 6,
     "A trait defines shared behavior; impl provides it for a concrete type.",
     "Make Meter.label() print 5 m.",
     "format! builds a String without printing it.",
     'trait Label { fn label(&self) -> String; }\nstruct Meter(i32);\nimpl Label for Meter {\n    fn label(&self) -> String {\n        // TODO: Format the value with its unit.\n        String::new()\n    }\n}\nfn main() {\n    println!("{}", Meter(5).label());\n}\n',
     "5 m", ["Read the tuple struct field with self.0.", 'Return format!("{} m", self.0).']),
    ("impl_default", "Create a default setting", 6,
     "Default provides a sensible initial value for a type.",
     "Make Settings::default() provide level 1.",
     "Return a Settings value from the trait method.",
     'struct Settings { level: u8 }\nimpl Default for Settings {\n    fn default() -> Self {\n        // TODO: Choose the initial level.\n        Self { level: 0 }\n    }\n}\nfn main() {\n    println!("{}", Settings::default().level);\n}\n',
     "1", ["Construct Self with the level field.", "Set level: 1 in default()."]),
    ("iterator_map", "Double a sequence", 6,
     "Iterator map transforms every item; sum combines the transformed values.",
     "Double every number and print their total.",
     "Call sum after map to produce one integer.",
     r('    let numbers = [1, 2, 3];\n    // TODO: Map each item to twice its value, then sum.\n    println!("0");'),
     "12", ["Start with numbers.iter().", "Use map(|n| n * 2).sum::<i32>()."]),
    ("option", "Use a missing-value fallback", 7,
     "Option<T> represents Some(value) or None; unwrap_or supplies a default.",
     "Print Guest when the optional name is absent.",
     "Do not unwrap a None value.",
     r('    let name: Option<&str> = None;\n    // TODO: Print a fallback for None.\n    println!("Replace me");'),
     "Guest", ["Call unwrap_or on name.", 'Print name.unwrap_or("Guest").']),
    ("result_question", "Propagate a parse error", 7,
     "The ? operator returns an Err early from a function that returns Result.",
     "Complete parse_number so valid input prints 42.",
     "The function's error type already matches parse().",
     'fn parse_number(text: &str) -> Result<i32, std::num::ParseIntError> {\n    // TODO: Parse text and propagate any error.\n    Ok(0)\n}\nfn main() {\n    println!("{}", parse_number("42").unwrap());\n}\n',
     "42", ["Call text.parse::<i32>()? inside the function.", "Store the result in value and return Ok(value)."]),
    ("filter", "Keep even values", 7,
     "filter retains iterator elements for which a predicate returns true.",
     "Count the even numbers and print the count.",
     "iter() lends references to the array elements.",
     r('    let numbers = [1, 2, 3, 4, 6];\n    // TODO: Filter even values and count them.\n    println!("0");'),
     "3", ["Start with numbers.iter().", "Use filter(|n| *n % 2 == 0).count()."]),
    ("collect", "Collect uppercase names", 7,
     "collect builds a collection from an iterator; map transforms each element first.",
     "Convert all names to uppercase and print the second one.",
     "Specify Vec<String> as the output collection type.",
     r('    let names = ["ada", "lin"];\n    // TODO: Collect uppercase names into a Vec<String> named upper.\n    println!("{}", names[1]);'),
     "LIN", ["Map each name with to_uppercase().", "Use let upper: Vec<String> = names.iter().map(|name| name.to_uppercase()).collect(); and print upper[1]."]),
    ("hashmap", "Count fruit names", 8,
     "HashMap entry gives access to a key's value and inserts a default when missing.",
     "Count the fruit names and print the count for apple.",
     "The entry API avoids a separate lookup before insertion.",
     'use std::collections::HashMap;\n' + r('    let fruits = ["apple", "pear", "apple", "apple"];\n    let mut counts: HashMap<&str, i32> = HashMap::new();\n    // TODO: Count every fruit.\n    println!("{}", counts.get("apple").unwrap_or(&0));'),
     "3", ["Loop over fruits and call counts.entry(fruit).", "Increment *counts.entry(fruit).or_insert(0) for each fruit."]),
    ("sort", "Find the median score", 8,
     "sort orders a mutable vector in ascending order.",
     "Sort the scores and print the middle value.",
     "A five-element vector has middle index 2.",
     r('    let mut scores = vec![9, 7, 2, 1, 5];\n    // TODO: Sort the vector.\n    println!("{}", scores[2]);'),
     "5", ["Vec has a sort() method.", "Call scores.sort() before printing."]),
    ("average", "Compute an integer average", 8,
     "The average of integer values can be computed by summing and dividing by their count.",
     "Calculate the average of the scores and print it.",
     "Convert len() from usize to i32 for the division.",
     r('    let scores = [10, 20, 30];\n    // TODO: Sum scores and divide by their count.\n    println!("0");'),
     "20", ["Use scores.iter().sum::<i32>().", "Divide the sum by scores.len() as i32."]),
    ("inventory", "Update stock counts", 8,
     "A map stores current stock; entry can update a key on every delivery.",
     "Process every delivery and print the stock of pens.",
     "The same item may appear more than once.",
     'use std::collections::HashMap;\n' + r('    let deliveries = [("pen", 2), ("book", 3), ("pen", 4)];\n    let mut stock: HashMap<&str, i32> = HashMap::new();\n    // TODO: Add each delivery to its item.\n    println!("{}", stock.get("pen").unwrap_or(&0));'),
     "6", ["Loop with for (item, amount) in deliveries.", "Use *stock.entry(item).or_insert(0) += amount;."]),
    ("gradebook", "Count passing students", 8,
     "Iterating over map values lets a gradebook count passing scores regardless of names.",
     "Print how many students scored at least 50.",
     "The pass threshold is inclusive.",
     'use std::collections::HashMap;\n' + r('    let grades = HashMap::from([("Ada", 80), ("Lin", 49), ("Mia", 50)]);\n    let mut passed = 0;\n    // TODO: Count grades of at least 50.\n    println!("{passed}");'),
     "2", ["Loop through grades.values().", "Increment passed when *score >= 50."]),
    ("report", "Build a compact report", 8,
     "A report combines filtering and formatting after calculating its summary.",
     "Count values above 10 and print high=<count>.",
     "Calculate high from values, then use the provided format string.",
     r('    let values = [4, 12, 15, 8, 20];\n    let mut high = 0;\n    // TODO: Count values above 10.\n    println!("high={high}");'),
     "high=3", ["Loop over values and compare each value with 10.", "Increment high for each value > 10."]),
])

# Extra cases vary the data supplied by the starter. They are distributed with
# the exercises, so they are additional cases rather than secret tests.
CASES = {
    "java": {
        "variables": [("different values", "int first = 12;", "int first = -4;", "4")],
        "condition": [("below boundary", "int age = 16;", "int age = 17;", "5"), ("boundary age", "int age = 16;", "int age = 18;", "10"), ("above boundary", "int age = 16;", "int age = 42;", "10")],
        "method": [("zero argument", "doubleValue(7)", "doubleValue(0)", "0"), ("negative argument", "doubleValue(7)", "doubleValue(-3)", "-6"), ("larger argument", "doubleValue(7)", "doubleValue(11)", "22")],
        "array": [("all negative", "{4, 9, 2, 7}", "{-8, -2, -11}", "-2")],
        "project": [("no discount", "{3, 7, 2}", "{3, 4, 2}", "9")],
        "average": [("uneven average", "{10, 20, 30}", "{5, 6}", "5")],
        "frequency": [("missing apple", '{"apple", "pear", "apple", "plum", "apple"}', '{"pear", "plum"}', "0")],
        "report": [("threshold boundary", "{4, 12, 15, 8, 20}", "{10, 11, 12}", "high=2")],
    },
    "rust": {
        "condition": [("below boundary", "let age = 16;", "let age = 17;", "5"), ("boundary age", "let age = 16;", "let age = 18;", "10"), ("above boundary", "let age = 16;", "let age = 42;", "10")],
        "function": [("zero argument", "double_value(7)", "double_value(0)", "0"), ("negative argument", "double_value(7)", "double_value(-3)", "-6"), ("larger argument", "double_value(7)", "double_value(11)", "22")],
        "vector": [("all negative", "vec![4, 9, 2, 7]", "vec![-8, -2, -11]", "-2")],
        "struct": [("different dimensions", "width: 3, height: 4", "width: 5, height: 6", "30")],
        "project": [("no discount", "vec![3, 7, 2]", "vec![3, 4, 2]", "9")],
        "average": [("uneven average", "[10, 20, 30]", "[5, 6]", "5")],
        "hashmap": [("missing apple", '["apple", "pear", "apple", "apple"]', '["pear", "plum"]', "0")],
        "report": [("threshold boundary", "[4, 12, 15, 8, 20]", "[10, 11, 12]", "high=2")],
    },
}


def generate(language, catalog, locale="fr"):
    if len(catalog) != 30:
        raise ValueError(f"{language} catalog must contain exactly 30 exercises")
    slugs = [item[0] for item in catalog]
    if len(set(slugs)) != len(slugs):
        raise ValueError(f"{language} catalog contains duplicate slugs")
    for slug, title, chapter, lesson, task, tip, template, output, hints in catalog:
        if not (1 <= chapter <= len(CHAPTERS)):
            raise ValueError(f"{language}_{slug}: invalid chapter")
        if not all((lesson, task, tip, template, output)) or len(hints) != 2:
            raise ValueError(f"{language}_{slug}: incomplete exercise")
        if "TODO" not in template:
            raise ValueError(f"{language}_{slug}: starter has no TODO")
        for name, old, _, _ in CASES[language].get(slug, []):
            if template.count(old) != 1:
                raise ValueError(f"{language}_{slug}: case {name!r} has no unique source anchor")
    directory = ROOT / language if locale == "fr" else ROOT / "en" / language
    directory.mkdir(parents=True, exist_ok=True)
    # The two known legacy naming schemes are the only files removed.
    for path in directory.glob("*.yaml"):
        if LEGACY.fullmatch(path.name) or GENERATED.fullmatch(path.name):
            path.unlink()
    for number, (slug, title, chapter, lesson, task, tip, template, output, hints) in enumerate(catalog, 1):
        key = f"{language}_{number:02d}_{slug}"
        if locale == "fr":
            title, lesson, task, tip, first_hint, second_hint = TEXTES[key]
            difficulty = "Débutant" if chapter <= 2 else "Intermédiaire"
            chapter_name = CHAPTERS[chapter - 1]
        else:
            title, lesson, task, tip, first_hint, second_hint = (
                title, lesson, task, tip, hints[0], hints[1]
            )
            difficulty = "Beginner" if chapter <= 2 else "Intermediate"
            chapter_name = CHAPTERS_EN[chapter - 1]
        record = {
            "id": key,
            "title": title,
            "language": language,
            "difficulty": difficulty,
            "chapter": f"{chapter:02d} · {chapter_name}",
            "lesson": lesson,
            "description": task,
            "tip": tip,
            "template": template,
            "expected_output": output,
            "hints": [first_hint, second_hint],
        }
        cases = CASES[language].get(slug, [])
        if cases:
            record["test_cases"] = [
                {"name": name, "replace": old, "with": new, "expected_output": expected}
                for name, old, new, expected in cases
            ]
        (directory / f"{number:02d}_{slug}.yaml").write_text(
            yaml.safe_dump(record, sort_keys=False, allow_unicode=True, width=90)
        )


if __name__ == "__main__":
    generate("java", JAVA)
    generate("rust", RUST)
    generate("java", JAVA, "en")
    generate("rust", RUST, "en")
    print(f"Generated {len(JAVA) + len(RUST)} curated exercises")
