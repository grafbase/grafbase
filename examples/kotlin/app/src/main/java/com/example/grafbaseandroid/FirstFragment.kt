package com.example.grafbaseandroid

import android.os.Bundle
import androidx.fragment.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.AdapterView
import android.widget.ArrayAdapter
import android.widget.ListView
import android.widget.Toast
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.whenStarted
import androidx.navigation.fragment.findNavController
import com.example.grafbaseandroid.Entities.Post
import com.example.grafbaseandroid.Repositories.PostRepository
import com.example.grafbaseandroid.databinding.FragmentFirstBinding
import kotlinx.coroutines.launch

/**
 * A simple [Fragment] subclass as the default destination in the navigation.
 */
class FirstFragment : Fragment() {

    private var _binding: FragmentFirstBinding? = null

    // This property is only valid between onCreateView and
    // onDestroyView.
    private val binding get() = _binding!!

    private var posts: List<Post> = mutableListOf()

    override fun onCreateView(
            inflater: LayoutInflater, container: ViewGroup?,
            savedInstanceState: Bundle?
    ): View? {

        _binding = FragmentFirstBinding.inflate(inflater, container, false)
        return binding.root

    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val arrayAdapter: ArrayAdapter<Post>
        val context = context as MainActivity

        var postsListView: ListView = binding.postsList
        arrayAdapter = ArrayAdapter(context, android.R.layout.simple_list_item_1, this.posts);
        postsListView.adapter = arrayAdapter

        postsListView.onItemClickListener = AdapterView.OnItemClickListener {
            parent, view, position, id ->
            val bundle = Bundle()
            val post = this.posts.get(position)
            bundle.putParcelable("post", post)
            findNavController().navigate(R.id.action_FirstFragment_to_SecondFragment, bundle);
        }

        lifecycleScope.launchWhenStarted {
            PostRepository().fetchPosts().onSuccess { result ->
                val posts = result.postCollection.edges.map { edge -> edge.node }
                arrayAdapter.clear()
                arrayAdapter.addAll(posts)
                arrayAdapter.notifyDataSetChanged()
            }.onFailure { exception ->
                exception.printStackTrace()
            }
        }
    }

    override fun onDestroyView() {
        super.onDestroyView()
        _binding = null
    }
}