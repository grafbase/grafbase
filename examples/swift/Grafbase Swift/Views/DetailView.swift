//
//  DetailView.swift
//  Grafbase Swift
//
//  Created by Craig Tweedy on 05/12/2022.
//

import SwiftUI

struct DetailView: View {
    var post: Post
    
    var body: some View {
        VStack(alignment: .leading) {
            Text(post.title).font(.subheadline)
                .padding(.bottom, 10)
            Spacer()
            if (post.comments?.edges ?? []).count > 0 {
                Text("Comments").font(.headline).padding(.bottom, 10)
                List((post.comments?.edges ?? []).map({$0.node}), id: \.id) { comment in
                    Text(comment.message)
                }
            }
        }
        .padding()
    }
}

struct DetailView_Previews: PreviewProvider {
    static var previews: some View {
        DetailView(
            post: Post(
                title: "Title"
            )
        )
    }
}
