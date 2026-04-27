// build.rs — Windows resource embedding
// Menambahkan icon aplikasi dan metadata ke binary (.exe) pada Windows.
// Dijalankan oleh Cargo sebelum kompilasi.
fn main() {
    // Hanya aktif di Windows
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();

        // Icon aplikasi (jika ada)
        if std::path::Path::new("./assets/icon.ico").exists() {
            res.set_icon("./assets/icon.ico");
        } else {
            eprintln!("cargo:warning=icon.ico tidak ditemukan — binary tanpa icon");
        }

        // Human readable metadata
        res.set("FileDescription", "App Blocker — Lab Computer Management");
        res.set("ProductName", "App Blocker");
        res.set("CompanyName", "Fahmi Dev");
        res.set(
            "LegalCopyright",
            "© 2026 Muhamad Fahmi. All rights reserved.",
        );
        res.set("OriginalFilename", "app_blocker.exe");
        res.set("InternalName", "App Blocker");
        res.set(
            "Comments",
            "App Blocker membantu pengelolaan komputer lab dengan fitur blocking dan monitoring.",
        );
        res.set("CompanyShortName", "FahmiDev");
        res.set("LegalTrademarks", "App Blocker™");
        res.set("PrivateBuild", "dev");
        res.set("SpecialBuild", "");

        // Versi numerik: major.minor.patch.build -> sebagai angka terpisah
        // Ambil dari Cargo env var dan parse
        let pkg_version = env!("CARGO_PKG_VERSION"); // e.g. "1.2.3"
        let mut ver_parts: Vec<u32> = pkg_version
            .split('.')
            .map(|s| s.parse::<u32>().unwrap_or(0))
            .collect();
        // pastikan ada 4 bagian (major, minor, patch, build)
        while ver_parts.len() < 4 {
            ver_parts.push(0);
        }
        let file_version = format!(
            "{}.{}.{}.{}",
            ver_parts[0], ver_parts[1], ver_parts[2], ver_parts[3]
        );
        // Set versi string dan numeric
        res.set("FileVersion", &file_version);
        res.set("ProductVersion", &file_version);
        res.set("FileVersion", &file_version);
        res.set("ProductVersion", &file_version);
        res.set("CompanyName", "Fahmi Dev");
        res.set("FileDescription", "App Blocker - Lab Computer Management");
        res.set("InternalName", "App Blocker");
        res.set("LegalCopyright", "Copyright 2026 Muhamad Fahmi");
        res.set("OriginalFilename", "app_blocker.exe");
        res.set("ProductName", "App Blocker");
        res.set("Comments", "Lab computer management");

        // Tambahkan additional string table entries (opsional)
        res.set("CompanyUrl", "https://github.com/mhmdfmi/app-blocker-rust");
        res.set(
            "SupportUrl",
            "https://github.com/mhmdfmi/app-blocker-rust/issues",
        );
        res.set("ContactInfo", "muhamadfahmi3240@gmail.com");

        // Manifest: request admin privileges dan DPI awareness
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.0.0.0"
    processorArchitecture="*"
    name="FahmiDev.AppBlocker"
    type="win32"
  />
  <description>App Blocker Lab Computer Management</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#);

        if let Err(e) = res.compile() {
            // Non-fatal: binary tetap bisa dikompilasi tanpa resource
            eprintln!("cargo:warning=Gagal compile resource Windows: {e}");
        }
    }

    // Rerun build.rs jika file ini atau icon berubah
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
