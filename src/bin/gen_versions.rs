use std::collections::BTreeMap;

fn main() {
    println!("#![cfg_attr(any(), rustfmt::skip)]");
    println!("// generated with `cargo run --bin gen_versions > src/versions/mod.rs`\n");

    let versions = [
        ("1.21.1", 767),
        ("1.21", 767),
        ("1.20.6", 766),
        ("1.20.5", 766),
        ("1.20.4", 765),
        ("1.20.3", 765),
        ("1.20.2", 764),
        ("1.20.1", 763),
        ("1.20", 763),
        ("1.19.4", 762),
        ("1.19.3", 761),
        ("1.19.2", 760),
        ("1.19.1", 760),
        ("1.19", 759),
        ("1.18.2", 758),
        ("1.18.1", 757),
        ("1.18", 757),
        ("1.17.1", 756),
        ("1.17", 755),
        ("1.16.5", 754),
        ("1.16.4", 754),
        ("1.16.3", 753),
        ("1.16.2", 751),
        ("1.16.1", 736),
        ("1.16", 735),
        ("1.15.2", 578),
        ("1.15.1", 575),
        ("1.15", 573),
        ("1.14.4", 498),
        ("1.14.3", 490),
        ("1.14.2", 485),
        ("1.14.1", 480),
        ("1.14", 477),
        ("1.13.2", 404),
        ("1.13.1", 401),
        ("1.13", 393),
        ("1.12.2", 340),
        ("1.12.1", 338),
        ("1.12", 335),
        ("1.11.2", 316),
        ("1.11.1", 316),
        ("1.11", 315),
        ("1.10.2", 210),
        ("1.10.1", 210),
        ("1.10", 210),
        ("1.9.4", 110),
        ("1.9.3", 110),
        ("1.9.2", 109),
        ("1.9.1", 108),
        ("1.9", 107),
        ("1.8.9", 47),
        ("1.8.8", 47),
        ("1.8.7", 47),
        ("1.8.6", 47),
        ("1.8.5", 47),
        ("1.8.4", 47),
        ("1.8.3", 47),
        ("1.8.2", 47),
        ("1.8.1", 47),
        ("1.8", 47),
        ("1.7.10", 5),
        ("1.7.9", 5),
        ("1.7.8", 5),
        ("1.7.7", 5),
        ("1.7.6", 5),
        ("1.7.5", 4),
        ("1.7.4", 4),
        ("1.7.3", 4),
        ("1.7.2", 4),
        ("1.7.1", 3),
        ("1.7", 3),
    ];

    let mut grouped_versions = BTreeMap::new();

    for version in versions {
        grouped_versions
            .entry(version.1)
            .or_insert(vec![])
            .insert(0, version.0);
    }

    for (proto_version, minecraft_versions) in &mut grouped_versions {
        let last_version = minecraft_versions.remove(minecraft_versions.len() - 1);

        if minecraft_versions.is_empty() {
            println!("/// for minecraft versions: {}", last_version);
        } else {
            println!(
                "/// for minecraft versions: {} & {}",
                minecraft_versions.join(", "),
                last_version
            );
        }

        println!("pub mod v{};", proto_version);
    }

    println!(
        "\npub use v{} as latest;",
        grouped_versions.last_entry().unwrap().key()
    );
}
