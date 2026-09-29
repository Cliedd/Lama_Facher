"""Generate the curated Forge curriculum. Requires PyYAML.

Run from any directory: python3 /path/to/generate_exercises.py
"""

from pathlib import Path
import re
import yaml


ROOT = Path(__file__).resolve().parent / "exercises"
LEGACY = re.compile(r"(?:\d{3}_(?:basic|loop|oop|control|structs)_\d+|0[12]_(?:hello_world|variables))\.yaml")
GENERATED = re.compile(r"\d{2}_[a-z]+\.yaml")
CHAPTERS = ["First steps", "Decisions and repetition", "Reusable code and data", "Real-world patterns"]


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


def generate(language, catalog):
    directory = ROOT / language
    directory.mkdir(parents=True, exist_ok=True)
    # The two known legacy naming schemes are the only files removed.
    for path in directory.glob("*.yaml"):
        if LEGACY.fullmatch(path.name) or GENERATED.fullmatch(path.name):
            path.unlink()
    for number, (slug, title, chapter, lesson, task, tip, template, output, hints) in enumerate(catalog, 1):
        record = {
            "id": f"{language}_{number:02d}_{slug}",
            "title": title,
            "language": language,
            "difficulty": "Beginner" if chapter <= 2 else "Intermediate",
            "chapter": f"{chapter:02d} · {CHAPTERS[chapter - 1]}",
            "lesson": lesson,
            "description": task,
            "tip": tip,
            "template": template,
            "expected_output": output,
            "hints": hints,
        }
        (directory / f"{number:02d}_{slug}.yaml").write_text(
            yaml.safe_dump(record, sort_keys=False, allow_unicode=True, width=90)
        )


if __name__ == "__main__":
    generate("java", JAVA)
    generate("rust", RUST)
    print(f"Generated {len(JAVA) + len(RUST)} curated exercises")
