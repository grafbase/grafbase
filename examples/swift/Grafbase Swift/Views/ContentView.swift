//
//  ContentView.swift
//  Grafbase Swift
//

import SwiftUI

struct ContentView: View {
    
    @State var posts: [Post] = []
    @State private var selectedPost: Post?
    let apiService: APIService = APIService()
    
    func loadPosts() async {
        self.posts = await self.apiService.listPosts()?.postCollection.edges.map({ $0.node }) ?? []
    }
    
    var body: some View {
        NavigationSplitView {
            List(self.posts, id: \.id, selection: $selectedPost) { post in
                NavigationLink(post.title, value: post)
            }
            .navigationTitle("Posts")
        } detail: {
            if let post = self.selectedPost {
                DetailView(post: post).navigationTitle(post.title)
            } else {
                Text("Select a post")
            }
        }
        .onAppear {
            Task.init {
                await self.loadPosts()
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView(posts: [
            Post(title: "Post 1"),
            Post(title: "Post 2"),
            Post(title: "Post 3"),
            Post(title: "Post 4"),
            Post(title: "Post 5"),
        ])
    }
}
