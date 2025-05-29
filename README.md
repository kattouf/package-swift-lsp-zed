# Package.swift LSP for Zed

A [Zed](https://zed.dev) extension that provides language server protocol (LSP) support for Swift Package Manager's Package.swift manifest files.

Powered by the [package-swift-lsp](https://github.com/kattouf/package-swift-lsp)

![demo](https://github.com/user-attachments/assets/4caa7126-a2d7-45dd-b663-2d3f31817f74)

> [!NOTE]
> For proper Swift language server functionality in non-Package.swift files, configure the language server order in your Zed settings:
> ```json
> {
>   "languages": {
>     "Swift": {
>       "language_servers": ["sourcekit-lsp", "package-swift-lsp"]
>     }
>   }
> }
> ```
