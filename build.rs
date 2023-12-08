use std::env;

fn main() {
    if env::var("CARGO_FEATURE_SSR").is_ok() {
        // Code to generate the file
        println!("Feature 'ssr' is enabled. Generating file...");
        use std::fs::File;
        use std::io::Write;
        use std::path::Path;
        // use ulid::Ulid;

        let version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set");
        let profile = env::var("PROFILE").expect("CARGO PROFILE NOT SET");

        let version = if matches!(profile.as_str(), "release") {
            version
        } else {
            return
        };

        use sailfish::TemplateOnce;

        #[derive(TemplateOnce)]
        #[template(path = "service-worker.js")]
        struct ServiceWorker {
            version: String,
        }

        let ctx = ServiceWorker { version };
        let str = ctx.render_once().expect("sw to generate");
        let path = Path::new("public/service-worker.js");
        let mut file = File::create(&path).expect("Could not create file");
        file.write_all(str.as_bytes()).expect("file to write");
        // ... file generation logic ...
    } else {
        println!("Feature 'ssr' is not enabled.");
    }
}
