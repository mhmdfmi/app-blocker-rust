/// build.rs — Windows resource embedding
/// Menambahkan icon aplikasi dan metadata ke binary (.exe) pada Windows.
/// Dijalankan oleh Cargo sebelum kompilasi.
fn main() {
    // Hanya aktif di Windows
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();

        // Icon aplikasi
        if std::path::Path::new("./assets/icon.ico").exists() {
            res.set_icon("./assets/icon.ico");
        } else {
            // Gunakan icon default jika icon.ico belum ada
            eprintln!("cargo:warning=icon.ico tidak ditemukan — binary tanpa icon");
        }

        // Metadata aplikasi (muncul di Properties → Details)
        res.set("FileDescription", "App Blocker — Lab Computer Management");
        res.set("ProductName", "App Blocker");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("CompanyName", "Fahmi Dev");
        res.set(
            "LegalCopyright",
            "© 2026 Muhamad Fahmi. All rights reserved.",
        );
        res.set("OriginalFilename", "app_blocker.exe");
        res.set("InternalName", "app_blocker");

        // Manifest: request admin privileges dan DPI awareness
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.3.5.0"
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
    println!("cargo:rerun-if-changed=icon.ico");
}
