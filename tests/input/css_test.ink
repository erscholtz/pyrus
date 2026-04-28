// Test CSS styling on document-level elements

document {
    @text(id="header", class="intro")[
        Welcome to the document
    ]
    @text(class="bodytext")[
        This is the body content
    ]
    @section(id="mainsection")[
        @text(class="nested")[
            Nested text in section
        ]
    ]
}

style {
    #header {
        font-size: 24pt;
        color: "blue";
    }

    .intro {
        font-weight: "bold";
    }

    .bodytext {
        font-size: 12pt;
        color: "black";
    }

    #mainsection {
        margin: 20pt;
        padding: 10pt;
    }

    .nested {
        font-style: "italic";
    }
}
