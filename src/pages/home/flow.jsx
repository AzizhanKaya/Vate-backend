import Post from '../../components/post';
import { get_time } from '@/wasm/wasm'
import { useState, useEffect } from "react"
export default function Flow({Topic}) {
  const [posts, setPosts] = useState([]);
  const timestamp = get_time().toString();


  useEffect(() => {
    async function get_posts() {
      try {
        const url = `http://192.168.1.25:3000/posts?sub=${Topic}&t=${timestamp}`;
        const response = await fetch(url, {
          method: 'GET'
        });

        if (response.ok) {
          const data = await response.json();
          setPosts(data);
        } else {
          console.log("Server error while getting posts:", await response.text());
        }
      } catch (e) {
        console.log("Error while getting posts:", e.message);
      }
    }

    get_posts();
  }, [Topic]);

  return (
    <>
      {posts.map((post, index) => <Post post={post} key={index} />)}
    </>
  );
}