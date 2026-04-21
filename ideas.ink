// general idea is @element_name (args, style=value) [ children ]

template {

    // custom elements

    func education_heading(
        institution: String, location: String,
        degree: String, minor: String,
        start_date: String, end_date: String
    ) {
        return @section [
            @text [ ${institution} ]
            @text [ ${location} ]
            @text [ ${degree} ]
            @text [ ${minor} ]
            @text [ ${start_date} - ${end_date} ]
            @children
        ]
    }
}

document {
    // headings

    @text (class="h1") [ Introduction ]           // Or @h1, @h2, etc.

    // text

    @text (class="body") [ This is content ]      // Text block, can be styled and accepts @break for formatting line breaks

    // lists

    @list [                        // List with content block
        - @text (id="first-item")[ First item ]               // block is for handling newline that should still be under the list
        - @text [ Second item ]
        - @text [ Third item ]
    ]

    // images

    @image (width=0.5) [ "path/to/photo.jpg" ]   // Image with attributes
    @image [ "path/to/photo.jpg" ]

    // then in use
    @education_heading (
        "University of Example", "Example City",
        "Bachelor of Science", "Minor in Technology",
        "2020", "2024"
    ) [
        @text[this is a regular text block inside the education_heading custom element]
    ]
}
