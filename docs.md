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
examples/lib/{library}.steel
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