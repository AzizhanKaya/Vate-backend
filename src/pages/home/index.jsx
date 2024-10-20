import StickyHeader from "../../components/sticky-header";
import Flow from "./flow";
import { useState } from "react";
export default function Home(){

    const [Topic, setTopic] = useState('Welcome');

    return(

        <>
            <StickyHeader title={Topic} />

            
            <div className="overflow-hidden z-1">
                <Flow Topic={Topic} />
            </div>
            
        </>
    )
}