//
//  GraphQLAPI.swift
//  Grafbase Swift
//

import Foundation

struct GraphQLOperation: Encodable {
    var operationString: String
    
    private var url = ""
    
    enum CodingKeys: String, CodingKey {
        case variables
        case query
    }
    
    init(_ operationString: String) {
        self.operationString = operationString
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(operationString, forKey: .query)
    }
    
    func getURLRequest() throws -> URLRequest {
        guard let url = URL(string: self.url), self.url != "" else {
            fatalError("Please fill in your URL")
        }
        var request = URLRequest(url: url)
        
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONEncoder().encode(self)
    
        return request
    }
}

class GraphQLAPI {
    
    func performOperation<Output: Decodable>(_ operation: GraphQLOperation) async throws -> Output {
        let request: URLRequest = try operation.getURLRequest()

        let (data, _) = try await URLSession.shared.getData(from: request)
        
        let result = try JSONDecoder().decode(GraphQLResult<Output>.self, from: data)
        guard let object = result.object else {
            print(result.errorMessages.joined(separator: "\n"))
            throw NSError(domain: "Error", code: 1)
        }
        
        return object
    }
}
