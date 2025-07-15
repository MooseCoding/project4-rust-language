#  Functions

Functions are defined like so

```
fun function_name(type arg1, ...args) {
    /* function body */
}
```

Functions can return a data type, or a custom class.

Functions can be used in the following manner

```
function_name(arg1, ...args);
```

# Variables

Variables are defined like so 

```
type variable_name = value;
```

And accessed like 

```
variable_name /* will be subbed for a value at runtime */
```

# If Statements

If statements are defined like so 

```
if (condition) {
    /* if body */ 
}
else {
    /* optional else body */ 
}
```

Conditions must evaluate to boolean expressions then the if body will run, else the optional else body will run. Note that there are no if else statements, you can embed if statements in other if statements or just simply not use them.

# While Loops 

While loops are defined like so 

```
while (condition) {
    /* while body */ 
}
```

Conditions must evaluate to boolean expressions, and while the condition is true, all of the code inside the body will be executed.

# For loops 

For loops are defined like so 

```
for (assignment; condition; increment) {
    /* for body */ 
}
```

Assignments must be a variable definition statement, conditions must be boolean expressions, and the increment will be run at the end of each loop. 

# Arrays 

Arrays are defined like so 

```
type[] array_name = [...values]; 
```

# Classes

Classes are defined like so

```
class Class_Name(type constructor_arg1, ...args) {
    /* class body */
}
```

Class can have variables and functions that are accessed in dot notation,
everything is public, private access is a maybe, but I prefer public only

And are used in the following manner

```
class_name.variable /* to access variable */
class_name.function(...args) /* to call a function */
```


# Libraries 

Libraries are all in a ./lib folder and are noted to be 
a library file with the usage of a .steel file
Like

```
./lib/{library}.steel
```

The standard library of math (and more tbd) can be included 
like 

```
import <math>;
```

and using non standard libraries is like

```
import "trig";
```