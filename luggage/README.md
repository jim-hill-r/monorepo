# Luggage

Portable Data Platform
(Private, Personal, Portable)

## Key Terms

Closet: This is like a user account. This contains all of your data (items), all your available bags and which cubes you would like to allow in your closet.  
Bag (Suitcase / Baggage): This is a piece of luggage that you would like to provide to someone. You can create as many of these as you want to send to folks. This can hold several packing cubes and each cube has permissions that the receiver will have. This also contains any fine grained permissions on items you may need.  
Cube (Packing Cube): This is a container for holding your data. It has a specific shape and size. For developers, think of this as a data object or struct. These are public schemas and can be shared between users and companies.
Bellhop: This is the service that handles what cubes are allowed in your closet, verifies who has access to what bags and manages the general flow of your data.  
Trolley: This is the current cubes you have with you. This is used in apps to manage what you need to fetch from the bellhop and send back to the closet. In technical terms, a trolley is the client side storage library allowing app devs to just define a struct and push it to luggage.

## API Documentation

The Bellhop API includes OpenAPI documentation powered by [utoipa](https://github.com/juhaku/utoipa).

When running the Bellhop service, you can access:
- **Swagger UI**: `http://localhost:3000/swagger-ui/` - Interactive API documentation
- **OpenAPI JSON**: `http://localhost:3000/api-docs/openapi.json` - Raw OpenAPI specification
