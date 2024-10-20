import { useState, useEffect } from 'react';
import Key from './key';


export default function Register({goToPage}) {
  const [copy, setCopy] = useState(false);
  const [nextButton, setNextButton] = useState(false);

  useEffect(() => {
    if (copy) {
      setNextButton(true);
    }
  }, [copy]);

  function handleNext(){
    if(copy){
      setNextButton(false);
      goToPage(2);
    }

  }
  

  return (
    <div className="flex flex-col items-center pt-[100px] w-full h-full" style={{ willChange: 'transform'}}>
      <div className="font-[Vate] p-3 text-[20px] mb-2">
        Hadi sana bir anahtar seçelim!
      </div>

      <div className="border-t-2 border-[#2f3336]">
        <Key setCopy={setCopy} />
      </div>

      

      <button
        className={`next-button ${nextButton ? 'fade-in' : 'fade-out'}`}
        onClick={handleNext}
      >
          Next
      </button>
    </div>
  );
}
