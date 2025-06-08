pub fn run() {
    println!("AUR submission is not directly supported in this tool.");
    println!("To submit a package to AUR, follow these steps:");
    println!("  1. Clone the AUR repository for your package:");
    println!("     git clone ssh://aur@aur.archlinux.org/<your-pkg>.git");
    println!("  2. Copy the PKGBUILD to your package's directory:");
    println!("     cp PKGBUILD <your-pkg>/");
    println!("  3. Navigate to the package directory and commit your changes:");
    println!("     cd <your-pkg> && git add PKGBUILD && git commit -m 'update' && git push");
    println!("For more details, visit: https://wiki.archlinux.org/title/Arch_User_Repository");
}

