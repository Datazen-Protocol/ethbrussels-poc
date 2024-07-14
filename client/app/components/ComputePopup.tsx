import React, { useState } from "react";
import Compute_handler from "../../../contract/compute_handler.json";
import { useWriteContract } from "wagmi";
import { parseEther } from "viem";
import {
  arbitrumSepolia,
  filecoinCalibration,
  polygonZkEvmCardona,
} from "viem/chains";

export default function ComputePopup({
  address,
  file_id,
}: {
  address: string;
  file_id: string;
}) {
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [value, setValue] = useState("");
  const [tval, setTval] = useState(0);
  const [chain, setChain] = useState("");
  const { writeContract } = useWriteContract();
  const [complete, setComplete] = useState(false);
  const [output, setOutput] = useState<any>();

  const toggleOpen = () => {
    setIsOpen((v) => !v);
  };

  const handleSelectionChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setValue(e.target.value);
  };
  const handleChainSelectionChange = (
    e: React.ChangeEvent<HTMLSelectElement>
  ) => {
    setChain(e.target.value);
  };

  const handleTVal = (e: React.ChangeEvent<HTMLInputElement>) => {
    console.log(Number(e.target.value));
    setTval(Number(e.target.value));
  };

  const startComputation = async () => {
    // send money onchain
    switch (chain) {
      case "filecoin":
        console.log(Compute_handler.filecoin);
        writeContract({
          abi: Compute_handler.abi,
          address: Compute_handler.filecoin as `0x${string}`,
          functionName: "requestCompute",
          value: parseEther("1"),
          chainId: filecoinCalibration.id,
        });
        break;
      case "arb":
        console.log(Compute_handler.arb);
        writeContract({
          abi: Compute_handler.abi,
          address: Compute_handler.arb as `0x${string}`,
          functionName: "requestCompute",
          value: parseEther("0.01"),
          chainId: arbitrumSepolia.id,
        });
        break;
      case "polygonZk":
        console.log(Compute_handler.polygonZk);
        writeContract({
          abi: Compute_handler.abi,
          address: Compute_handler.polygonZk as `0x${string}`,
          functionName: "requestCompute",
          value: parseEther("0.01"),
          chainId: polygonZkEvmCardona.id,
        });
        break;

      default:
        throw Error("Invalid Chain");
    }
    // api call for compute
    let headersList = {
      Accept: "*/*",
      "Content-Type": "application/json",
    };

    let bodyContent: {
      address: string;
      filename: string;
      compute_type: string;
      threshold: number | undefined;
      chain: string;
    } = {
      address: address,
      filename: file_id,
      compute_type: value,
      threshold: tval,
      chain,
    };
    console.log(bodyContent);
    let response = await fetch("http://localhost:8000/compute", {
      method: "POST",
      body: JSON.stringify(bodyContent),
      headers: headersList,
    });

    let data = await response.text();
    setOutput(JSON.parse(data));
    setComplete(true);
    console.log(data);
  };
  return (
    <div>
      <button
        className="flex-end border-2 border-[#A5D6FF] p-3 rounded-xl"
        onClick={toggleOpen}
      >
        Compute
      </button>
      {isOpen && (
        <div className="fixed inset-0 z-50 bg-black bg-opacity-40 flex flex-col justify-center items-center p-4">
          <div className="relative bg-white rounded-lg p-4 w-full max-w-[80%] overflow-hidden border-1">
            {complete ? (
              <div className="flex flex-col space-y-5">
                <h1 className="text-xl font-semibold">Compute Results</h1>
                <h2 className="text-md">Computation Proof: </h2>
                <textarea className="w-[80%]">{output?.proof}</textarea>
                <h2 className="text-md">Computation Output: </h2>
                <textarea className="w-[80%]">
                  {output?.compute_result}
                </textarea>
              </div>
            ) : (
              <div className="flex flex-col space-y-5">
                <h1 className="text-xl font-semibold">
                  Compute on {address}/{file_id}
                </h1>
                <select
                  id="Chain"
                  className="border border-[#A5D6FF] bg-[#d0f1ff] text-sm rounded-lg focus:outline-none block w-full p-2.5 "
                  onChange={handleChainSelectionChange}
                >
                  <option selected value="">
                    Choose Chain
                  </option>
                  <option value="filecoin">
                    Filecoin - Calibration testnet
                  </option>
                  <option value="arb">Arbitrum - Sepolia testnet</option>
                  <option value="polygonZk">
                    Polygon zkEVM - Cardona Testnet
                  </option>
                </select>
                <select
                  id="enc_type"
                  className="border border-[#A5D6FF] bg-[#d0f1ff] text-sm rounded-lg focus:outline-none block w-full p-2.5 "
                  onChange={handleSelectionChange}
                >
                  <option selected value="">
                    Choose Compute Type
                  </option>
                  <option value="Total">Total i.e. Sum of all values</option>
                  <option value="GT">Total Greater Than</option>
                  <option value="GE">Total Greater Than And Equal</option>
                  <option value="LT">Total Less Than</option>
                  <option value="LE">Total Less Than And Equal</option>
                </select>
                {value == "Total" || "" ? (
                  <></>
                ) : (
                  <div>
                    <label
                      htmlFor="Threshold Value"
                      className="block mb-2 text-md font-normal"
                    >
                      Threshold Value
                    </label>
                    <input
                      id="TVAL"
                      type="number"
                      className="border border-[#A5D6FF] bg-[#d0f1ff] text-sm rounded-lg focus:outline-none block w-full p-2.5 "
                      onChange={handleTVal}
                      placeholder="Threshold Value"
                    />
                  </div>
                )}
                <button
                  className="border-2 border-[#A5D6FF] p-3 w-max rounded-md"
                  onClick={startComputation}
                >
                  Start FHE Computation
                </button>
              </div>
            )}
            <button onClick={toggleOpen} className="border-2 border-red-300 bg-transparent p-3 rounded-md mt-3">Close</button>
          </div>
        </div>
      )}
    </div>
  );
}
