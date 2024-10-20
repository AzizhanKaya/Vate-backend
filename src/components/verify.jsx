import { useState } from 'react';
import { Transition } from '@headlessui/react';

const Verify = () => {
  const [showText, setShowText] = useState(false);

  return (
    <div
      className="ml-auto relative"
      onMouseEnter={() => setShowText(true)}
      onMouseLeave={() => setShowText(false)}
    >
      <svg
        width="15"
        height="15"
        viewBox="8 8 84 84"
        aria-label="Verification check"
      >
        <circle cx="50" cy="50" r="40" strokeWidth="3" fill="#05995b" />
        <path d="M30 50 L45 65 L70 35" stroke="white" strokeWidth="9" fill="none" />
      </svg>

      <Transition
        show={showText}
        enter="transition duration-200 ease-out"
        enterFrom="transform opacity-0"
        enterTo="transform opacity-100"
        leave="transition duration-200 ease-out"
        leaveFrom="transform opacity-100"
        leaveTo="transform opacity-0"
      >
        <div className="absolute text-center z-100 bottom-full mb-1 left-1/2 transform -translate-x-1/2 bg-[#090a0ac7] border border-[#2f3336] text-[#9fa3a7] text-sm rounded-l p-1">
          Signature verified.
        </div>
      </Transition>
    </div>
  );
};

export default Verify;
