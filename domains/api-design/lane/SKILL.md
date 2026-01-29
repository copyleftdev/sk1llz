---
name: lane-api-evangelist
description: Design APIs using Kin Lane's "API Evangelist" philosophy. Emphasizes Design-First (OpenAPI), governance, treating APIs as products, and the political/business impact of interfaces. Use when building public platforms or large-scale internal ecosystems.
---

# Kin Lane Style Guide

## Overview

Kin Lane (The API Evangelist) champions the idea that **APIs are products**, not just technical pipes. His philosophy centers on **Design First**: defining the contract (OpenAPI) before writing a single line of code. This ensures stakeholders agree on the interface, enables parallel development (mocking), and enforces governance.

> "The API contract is the truth. The code is just an implementation detail."

## Core Principles

1.  **Design First, Code Second**: Always start with an OpenAPI Specification (OAS). If you are writing a controller or a route handler before you have a YAML spec, you are doing it wrong.
2.  **API as a Product**: Your API has users (developers). It needs documentation, support, a roadmap, and a value proposition.
3.  **Governance & consistency**: Use linting (Spectral) to enforce style guides across all APIs in an organization. Consistency breeds usability.
4.  **Human-Readable Contracts**: Descriptions in your OAS are not optional. They are the primary documentation.
5.  **Mocking**: Use your design to generate mock servers immediately. Get feedback from the frontend team before the backend is built.

## Prompts

### Design a New API

> "Act as Kin Lane. Design a RESTful API for [Domain]. Start by creating a comprehensive OpenAPI 3.1 definitions. Do not write implementation code yet.
>
> Focus on:
> 1.  **Resource Design**: Nouns over verbs (e.g., `/users`, not `/getUsers`).
> 2.  **Standard Status Codes**: Use the full HTTP spectrum (201, 202, 204, 400, 401, 403, 404, 409, 429).
> 3.  **Problem Details**: Use RFC 7807 for error responses.
> 4.  **Descriptions**: Every schema, parameter, and endpoint must have a verbose, helpful description.
> 5.  **Reusability**: Use `$ref` for shared components."

### Audit an Existing API

> "Critique this API design from the perspective of the API Evangelist.
>
> Look for:
> *   **Leaky Abstractions**: Database columns exposed directly in the API.
> *   **Inconsistency**: Different naming conventions (camelCase vs snake_case) or path structures.
> *   **Missing Metadata**: Lack of descriptions, examples, or contact info in the spec.
> *   **Governance Violations**: Does it follow standard REST practices? Is it versioned correctly?"

## Examples

### The OpenAPI Contract (The Source of Truth)

```yaml
openapi: 3.1.0
info:
  title: BookStore Platform API
  version: 1.0.0
  description: |
    The central interface for the BookStore ecosystem. 
    This API allows partners to manage inventory, process orders, and track shipments.
    
    ## Authentication
    All requests must include a valid API Key in the `X-API-Key` header.
  contact:
    name: API Governance Team
    email: api@bookstore.com
    url: https://developer.bookstore.com/support
  license:
    name: Apache 2.0
    url: https://www.apache.org/licenses/LICENSE-2.0.html
tags:
  - name: Inventory
    description: Operations related to book stock and warehouses.
  - name: Orders
    description: Lifecycle management for customer orders.

paths:
  /books:
    get:
      summary: List all books
      operationId: listBooks
      tags: [Inventory]
      description: |
        Retrieve a paginated list of books. 
        Supports filtering by author, genre, and publication date.
      parameters:
        - name: limit
          in: query
          description: Maximum number of items to return.
          schema:
            type: integer
            default: 20
            maximum: 100
        - name: page
          in: query
          schema:
             type: integer
             default: 1
      responses:
        '200':
          description: A list of books
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BookList'
        '429':
          $ref: '#/components/responses/RateLimited'
    post:
      summary: Add a new book
      operationId: createBook
      tags: [Inventory]
      description: Register a new book in the system. Requires `write:inventory` scope.
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Book'
      responses:
        '201':
          description: Book successfully created. Location header contains the URL of the new resource.
          headers:
            Location:
              description: URL of the created resource
              schema:
                type: string
        '400':
          $ref: '#/components/responses/BadRequest'

components:
  schemas:
    Book:
      type: object
      required: [isbn, title, author_id]
      properties:
        id:
          type: string
          format: uuid
          readOnly: true
        isbn:
          type: string
          pattern: '^(?=(?:\D*\d){10}(?:(?:\D*\d){3})?$)[\d-]+$'
          example: "978-3-16-148410-0"
        title:
          type: string
          example: "The API Design Guide"
        price:
          description: Price in cents
          type: integer
          minimum: 0
    BookList:
      type: object
      properties:
        data:
          type: array
          items:
            $ref: '#/components/schemas/Book'
        meta:
          $ref: '#/components/schemas/PaginationMeta'

  responses:
    BadRequest:
      description: The server could not understand the request due to invalid syntax.
      content:
         application/problem+json:
            schema:
              $ref: '#/components/schemas/ProblemDetails'
    RateLimited:
      description: You have exceeded your rate limit.
      headers:
        Retry-After:
          description: The number of seconds to wait before making a new request.
          schema:
            type: integer

```

### Anti-Patterns (What NOT to do)

*   **Code First**: Writing a Python/Go struct and auto-generating the Swagger. This makes the API implementation-dependent and often ugly.
*   **Database Driven**: Exposing `created_at`, `updated_at`, or internal `user_id` fields that have no business meaning to the consumer.
*   **Vague Errors**: Returning `500 Internal Server Error` with `{"error": "Something went wrong"}`. Always use RFC 7807 (`type`, `title`, `detail`, `instance`).
*   **Breaking Changes**: Changing a field name or type without bumping the API version (v1 -> v2).

## Resources

*   [API Evangelist Blog](https://apievangelist.com/)
*   [OpenAPI Specification](https://www.openapis.org/)
*   [Spectral (JSON/YAML Linter)](https://stoplight.io/open-source/spectral)
