# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = "activestorage-ocr"
  spec.version       = "0.1.0"
  spec.authors       = ["Your Name"]
  spec.email         = ["your@email.com"]

  spec.summary       = "OCR support for Rails Active Storage"
  spec.description   = "Extract text from images and PDFs stored in Active Storage using a high-performance Rust OCR server"
  spec.homepage      = "https://github.com/Cause-of-a-Kind/activestorage-ocr"
  spec.license       = "MIT"
  spec.required_ruby_version = ">= 3.2.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = "#{spec.homepage}/blob/main/CHANGELOG.md"

  spec.files = Dir.chdir(__dir__) do
    Dir["{lib}/**/*", "LICENSE", "README.md"]
  end

  spec.require_paths = ["lib"]

  spec.add_dependency "activestorage", ">= 7.0"
  spec.add_dependency "faraday", ">= 2.0"
  spec.add_dependency "faraday-multipart", ">= 1.0"

  spec.add_development_dependency "minitest", "~> 5.0"
  spec.add_development_dependency "webmock", "~> 3.0"
  spec.add_development_dependency "rake", "~> 13.0"
end
