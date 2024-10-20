import React, { useState } from "react";
import { slides } from "./slides";
import "./slider.css";

const Slider = ({setExitModal}) => {
  const [currentPage, setCurrentPage] = useState(0);
  const [nextPage, setNextPage] = useState(null);
  const [isSliding, setSliding] = useState(false);

  function goToPage(pageIndex) {
    setNextPage(pageIndex);
    setSliding(true);
  };

  
  function handleAnimationEnd(event) {
    if (nextPage === null) return;
    if (event.animationName !== "slide-out") return;
    setCurrentPage(nextPage);
    setNextPage(nextPage+1);
    setSliding(false);
  };

  

  const CurrentComponent = slides[currentPage];
  const NextComponent = slides[nextPage];

  return (
    <div className="slider w-full h-full p-16 flex justify-center items-center">
      
        <div
          className="absolute w-full h-full"
          style={{
            animation: isSliding ? 'slide-out 0.5s forwards' : '',
            willChange: 'transform'
          }}
          onAnimationEnd={handleAnimationEnd}
        >
          <CurrentComponent
            goToPage={goToPage}
            setExitModal={setExitModal}
          />
        </div>

        {NextComponent && (
          <div
            className="absolute next-page w-full h-full"
            style={{
              animation: isSliding ? 'slide-in 0.5s forwards' : '',
              willChange: 'transform'
            }}
          >
            <NextComponent/>
          </div>
        )}
    </div>
  );
};

export default Slider;