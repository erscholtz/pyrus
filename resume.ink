template {
    title = "Erik Scholtz Resume"
    author = "Erik Scholtz"
    font_size = 10

    func resume_header(
        name: String,
        target_role: String,
        email: String,
        linkedin: String,
        github: String
    ) {
        return @section(class="resume_header")[
            @text(class="name")[${name}]
            @text(class="target_role")[${target_role}]
            @text(class="contact")[${email} | linkedin.com/in/${linkedin} | github.com/${github}]
        ]
    }

    func resume_section(title: String) {
        return @section(class="resume_section")[
            @text(class="section_title")[${title}]
            @separator(class="section_rule")
            @children
        ]
    }

    func skills_block() {
        return @list(class="skills")[
            - @text[Languages:      Rust, C, C++, Python]
            - @text[Systems:        Linux kernel drivers, UEFI/EDK-II, QEMU, TPM2, IPC, JSON-RPC]
            - @text[Tools:          Cargo, CMake, KBuild, Make, Meson, Git, Valgrind, SQLite]
        ]
    }

    func project_link(url: String, label: String) {
        return @link(class="project_link")["${url}", "${label}"]
    }

    func project_links(url: String) {
        return @section(class="project_links")[
            @project_link("${url}", "github")
        ]
    }

    func project_entry(title: String, stack: String, url: String) {
        return @section(class="project")[
            @section(class="entry_header")[
                @text(class="entry_title")[${title}]
                @project_links("${url}")
            ]
            @text(class="project_stack")[${stack}]
            @children
        ]
    }

    func experience_entry(
        role: String,
        company: String,
        location: String,
        start_date: String,
        end_date: String
    ) {
        return @section(class="experience")[
            @section(class="entry_header")[
                @text(class="entry_title")[${role}]
                @text(class="entry_side")[${start_date} – ${end_date}]
            ]
            @text(class="entry_subtitle")[${company} · ${location}]
            @children
        ]
    }

    func education_entry(
        institution: String,
        location: String,
        degree: String,
        distinction: String,
        start_date: String,
        end_date: String
    ) {
        return @section(class="education")[
            @section(class="entry_header")[
                @text(class="entry_title")[${institution}]
                @text(class="entry_side")[${start_date} – ${end_date}]
            ]
            @text(class="degree")[${degree}]
            @text(class="entry_subtitle")[${location}]
            @text(class="degree_detail")[${distinction}]
            @children
        ]
    }
}

document {
    @resume_header(
        "Erik Scholtz",
        "Systems and Infrastructure Engineer",
        "erikscholtz23@gmail.com",
        "erikscholtz",
        "erscholtz"
    )

    @resume_section("Skills")[
        @skills_block()
    ]

    @resume_section("Projects")[
        @project_entry(
            "harnessd Autocomplete Daemon",
            "Rust · Tokio · tree-sitter · SQLite · JSON-RPC",
            "github.com/erscholtz/harnessd"
        )[
            @list(class="bullets")[
                - @text[Engineered a Rust daemon with local IPC, lockfile-based single-instance control, and graceful shutdown to serve editor and CLI requests over JSON-RPC 2.0.]
                - @text[Built a rusqlite proposal cache keyed by file path, byte range, and content hash, using indexed lookups and invalidation guards to keep completions fast, bounded, and coherent with source changes.]
                - @text[Used tree-sitter to resolve cursor byte offsets to AST regions and precompute cached suggestions from TODO/FIXME anchors across multiple languages.]
            ]
        ]

        @project_entry(
            "UEFI CVE Replication and Firmware Patch",
            "C · EDK-II · QEMU · UEFI Secure Boot",
            "github.com/erscholtz/uefi-cve-replication"
        )[
            @list(class="bullets")[
                - @text[Replicated CVE-2022-21894 (Baton Drop) using QEMU; demonstrated vulnerability by safely sandboxing exploit payload in QEMU to understand attack vector.]
                - @text[Proposed proof-of-concept patch through monotonic counter in EDK-II using C on EFI file sign date timestamps preventing rollback attacks.]
            ]
        ]

        @project_entry(
            "Pyrus Domain-Specific Compiler",
            "Rust · Recursive Descent Parsing · HIR · PDF Rendering",
            "github.com/erscholtz/pyrus"
        )[
            @list(class="bullets")[
                - @text[Zero-dependency compiler pipeline from lexer to parser to HIR for a typesetting language. Created this resume.]
                - @text[Implemented recursive descent parser with Pratt-style operator precedence and string interpolation support.]
            ]
        ]

        @project_entry(
            "Linux Character Device Driver",
            "C · KBuild · Make · Linux Kernel APIs",
            "github.com/erscholtz/linux-character-device-driver"
        )[
            @list(class="bullets")[
                - @text[Implemented file operations using kernel APIs register_chrdev, class_create, and device_create.]
                - @text[Enabled dynamic runtime configuration through write operations allowing users to change output byte values.]
                - @text[Utilized kernel memory management functions copy_from_user and put_user for user-kernel space data transfer.]
                - @text[Built and deployed using kernel build system KBuild with Makefile automation.]
            ]
        ]
    ]

    @resume_section("Experience")[
        @experience_entry(
            "Infrastructure Engineer",
            "Deloitte",
            "Toronto, ON",
            "September 2025",
            "Present"
        )[
            @list(class="bullets")[
                - @text[Applied constraint-solving techniques to optimize CI resource scheduling, reducing queue times by 20% under shared compute constraints.]
            ]
        ]

        @experience_entry(
            "Infrastructure Engineer",
            "Syncademia AI",
            "Waterloo, ON",
            "January 2025",
            "August 2025"
        )[
            @list(class="bullets")[
                - @text[Reduced dependency surface area by 40% through targeted library audit and replacement, simplifying runtime behavior and eliminating classes of failure.]
                - @text[Built a streaming document ingestion pipeline with backpressure handling to process 1000+ page inputs without exceeding memory limits.]
            ]
        ]

        @experience_entry(
            "Software Engineering Co-op",
            "Deloitte",
            "Toronto, ON",
            "May 2024",
            "September 2024"
        )[
            @list(class="bullets")[
                - @text[Developed a telemetry tool to analyze system logs and visualize object dependencies, adopted by multiple enterprise teams for debugging.]
            ]
        ]
    ]

    @resume_section("Education")[
        @education_entry(
            "Carleton University",
            "Ottawa, ON",
            "Bachelor of Computer Science",
            "Honours with Distinction - 3.7 GPA",
            "September 2020",
            "December 2024"
        )[
            @list(class="bullets")[
                - @text[Mathematics Minor: Linear Programming Optimizations, Computational Numerical Methods]
                - @text[Honours Project: UEFI CVE Replication and Firmware Hardening, co-authored with peers]
                - @text[Selected Coursework: Operating Systems, Distributed Systems, Trusted Computing, Programming Paradigms]
            ]
        ]
    ]
}

style {
    body {
        font-family: "Georgia";
        font-size: 10.25pt;
        line-height: 1.16;
        margin: 0.38in;
    }

    .resume_header {
        margin-bottom: 1.5pt;
        padding-bottom: 0pt;
    }

    .name {
        font-size: 20.5pt;
        font-weight: 700;
        line-height: 1.05;
        margin-bottom: 1pt;
    }

    .target_role {
        font-size: 10.75pt;
        font-weight: 700;
        margin-top: 1pt;
    }

    .contact {
        font-size: 9.25pt;
        margin-top: 2pt;
    }

    .resume_section {
        margin-top: 6pt;
    }

    .section_title {
        font-size: 10.8pt;
        font-weight: 800;
        line-height: 1.05;
        margin-bottom: 0.8pt;
        padding-bottom: 0pt;
    }

    .section_rule {
        height: 0.45pt;
        margin-top: 0pt;
        margin-bottom: 2pt;
    }

    .entry_header {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: baseline;
        column-gap: 10pt;
        margin-bottom: 0.7pt;
    }

    .entry_title {
        font-size: 10.35pt;
        font-weight: 700;
        line-height: 1.08;
    }

    .entry_side {
        width: 130pt;
        font-size: 8.9pt;
        font-weight: 700;
        line-height: 1.08;
        white-space: nowrap;
        text-align: right;
    }

    .entry_subtitle {
        font-size: 8.9pt;
        font-weight: 600;
        margin-top: 0.7pt;
        margin-bottom: 1.5pt;
    }

    .project_stack {
        font-size: 8.9pt;
        font-weight: 600;
        margin-top: 0.7pt;
        margin-bottom: 1.5pt;
    }

    .project_links {
        width: 130pt;
        display: flex;
        flex-direction: row;
        justify-content: flex-end;
        column-gap: 3pt;
        white-space: nowrap;
    }

    .project_link {
        display: block;
        font-size: 8.3pt;
        font-weight: 700;
        line-height: 1;
        white-space: nowrap;
        text-decoration: none;
    }

    .project,
    .experience,
    .education {
        margin-top: 3.5pt;
        break-inside: avoid;
        page-break-inside: avoid;
    }

    .degree {
        font-weight: 700;
    }

    .degree_detail {
        font-size: 9.25pt;
    }

    .skills {
        margin-top: 1pt;
        margin-bottom: 2pt;
        padding-left: 0pt;
        marker-width: 0pt;
        marker-gap: 0pt;
        list-style-type: none;
    }

    .bullets {
        margin: 1.5pt;
        padding-left: 12pt;
        marker-width: 5pt;
        marker-gap: 3pt;
    }

    strong {
        font-weight: 700;
    }
}
