package com.example.grafbaseandroid

import android.R
import android.os.Bundle
import androidx.fragment.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ArrayAdapter
import android.widget.ListView
import com.example.grafbaseandroid.Entities.Comment
import com.example.grafbaseandroid.Entities.Post
import com.example.grafbaseandroid.databinding.FragmentSecondBinding

/**
 * A simple [Fragment] subclass as the second destination in the navigation.
 */
class SecondFragment : Fragment() {

    private var _binding: FragmentSecondBinding? = null

    // This property is only valid between onCreateView and
    // onDestroyView.
    private val binding get() = _binding!!

    override fun onCreateView(
            inflater: LayoutInflater, container: ViewGroup?,
            savedInstanceState: Bundle?
    ): View {

        _binding = FragmentSecondBinding.inflate(inflater, container, false)
        return binding.root

    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val arrayAdapter: ArrayAdapter<Comment>
        val context = context as MainActivity

        var commentsListView: ListView = binding.comments
        arrayAdapter = ArrayAdapter(context, R.layout.simple_list_item_1, mutableListOf());
        commentsListView.adapter = arrayAdapter

        arguments?.getParcelable<Post>("post").let { post ->
            binding.title.text = post?.title
            binding.content.text = post?.body

            val comments = post?.comments?.edges?.map { edge -> edge.node }.orEmpty()
            if (comments.isEmpty()) {
                binding.commentsTitle.visibility = View.INVISIBLE;
                binding.comments.visibility = View.INVISIBLE;
            } else {
                arrayAdapter.clear()
                arrayAdapter.addAll(comments)
                arrayAdapter.notifyDataSetChanged();
            }
        }
    }

    override fun onDestroyView() {
        super.onDestroyView()
        _binding = null
    }
}