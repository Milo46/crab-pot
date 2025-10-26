# Documentation

This directory contains the technical documentation for the Log Server project.

## 📁 Files

* **[`SRD.md`](./SRD.md)** - Software Requirements Document with complete technical specification
* **[`openapi.yaml`](./openapi.yaml)** - OpenAPI 3.0 API specification with interactive documentation support

## 🔍 Viewing the API Documentation

### Interactive Documentation

1. **Swagger Editor**: Upload [`openapi.yaml`](./openapi.yaml) to [editor.swagger.io](https://editor.swagger.io/)
2. **VS Code**: Install "OpenAPI (Swagger) Editor" extension
3. **Local Swagger UI**: `npx swagger-ui-serve openapi.yaml`
4. **Redoc**: `npx redoc-cli serve openapi.yaml`

### Using the OpenAPI Spec

* Generate client SDKs for various programming languages
* Import into API testing tools (Postman, Insomnia)
* Create mock servers for development
* Validate API responses and contracts

## 🔄 Documentation Workflow

When making API changes:

1. **Update OpenAPI spec first** - Design-first approach
2. **Validate YAML syntax** - Ensure specification is valid
3. **Update SRD if needed** - Keep architectural docs in sync
4. **Maintain consistency** - Examples should match across documents
5. **Version appropriately** - Update version numbers

## 📖 Key Documentation Sections

### Software Requirements Document (SRD)

* Functional & non-functional requirements
* Schema-based architecture design
* Database models and API specifications
* Technology stack and deployment details
* Future roadmap and improvements

### OpenAPI Specification

* Complete API endpoint definitions
* Request/response schemas with validation
* Error handling specifications
* Interactive examples and testing support

## 🛠️ Documentation Tools

* [OpenAPI Specification](https://spec.openapis.org/oas/v3.0.3) - API documentation standard
* [Swagger Tools](https://swagger.io/tools/) - OpenAPI toolchain
* [Redoc](https://redoc.ly/) - Beautiful API documentation
* [JSON Schema](https://json-schema.org/) - Schema validation specification
