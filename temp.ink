// Sample PDF markdown test file

template {
    title = "My Document"       // this is a default variable for the document
    author = "Alice"            // this is a default variable for the document
    font_size = 12              // this is a default variable for the document

    // Simple formula
    let total_price = "$price * quantity$" // this is not a default value and needs to be defined with "let" or "const"

    const tax_rate = 0.08       // this is a constant value that can be used throughout the document

    func intro_section(param1: String, param2: Int) { // functions called in document section must be of type DocElement
        return text {"introduction, the total price is {total_price}"}
    }

    func more_complex_section() { // docElement returned here
        return section {
            text {
                "This is a more complex section"
            }
            text {
                "This is another text element"
            }
            text (id="listelement"){
                "This is a third text element"
            }
            list {
                item {
                    "Item 1"
                }
                item {
                    "Item 2"
                }
                item {
                    "Item 3"
                }
            }
        }
    }
}

document { // document cannot have variable declared in it
    intro_section("name", 41) // section has default attributes that can be called
    text {
        this is also text that can be parsed by the compiler
    }
    list {
        item {
            "Item 0"
        }
        item {
            "Item 1"
        }
        item {
            "Item 2"
        }
    }

    more_complex_section()
}

style {
    body {
        font-family: "Helvetica";
        color: "black";
        margin: 1pt;
    }

    .intro, .more_complex_section {
        font-size: 23pt;          /* overloaded font size */
        font-weight: "bold";    /* overloaded entire section styling */
    }

    #listelement {
        font-size: 18pt;
        font-weight: "normal";
    }
}

/* End of test file */
