"use client";
import React, { Dispatch, SetStateAction, useState } from "react";
import { Divider } from "@nextui-org/divider";
import {
  Listbox,
  ListboxItem,
  Radio,
  RadioGroup,
  Select,
  SelectItem,
} from "@nextui-org/react";
import { useAccount } from "wagmi";

const page = () => {
  const { address, isConnected } = useAccount();
  const [value, setValue] = useState("");
  const [description, setDescription] = useState("");
  const [fileID, setFileID] = useState("");
  const [file, setFile] = useState<File | null>(null);
  const [status, setStatus] = useState<0 | 1 | 2>(0);
  const [ipfsHash, setIpfsHash] = useState("");

  const handleSelectionChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setValue(e.target.value);
  };

  const handleDesChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setDescription(e.target.value);
  };
  const handleLabelChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFileID(e.target.value);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      setFile(e.target.files[0]);
    }
  };

  const submitForm = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    console.log({ value, description, fileID, address });
    setStatus(1);
    if (file) {
      const formData = new FormData();
      formData.append("data", file);
      formData.append("address", address?.toString() ?? "");
      formData.append("filename", value == "FHE" ? `fhe_${fileID}` : fileID);
      formData.append("description", description);
      let resp = await fetch("http://localhost:8000/store", {
        method: "POST",
        body: formData,
        headers: {
          Accept: "*/*",
        },
      });
      let data = await resp.text();
      let hash = parseResponse(data);
      setIpfsHash(hash);
      setStatus(2);
    }
  };

  return (
    <div className="py-16 mb-5 px-20 space-y-[30px] w-full">
      <h1 className="text-[50px] font-[500]">Store Data</h1>
      <Divider className="my-4 border-[#11023B] border-1 w-full" />
      <form className="my-5 w-full space-y-8" onSubmit={submitForm}>
        <div>
          <label htmlFor="Title" className="block mb-2 text-md font-normal">
            File Identifier
          </label>
          <input
            id="Title"
            type="text"
            className="border border-[#A5D6FF] bg-[#d0f1ff] text-sm rounded-lg focus:outline-none block w-full p-2.5 "
            onChange={handleLabelChange}
            placeholder="Data Title"
          />
        </div>
        <div>
          <label htmlFor="enc_type" className="block mb-2 text-md font-normal">
            Select a File Encryption Type
          </label>
          <select
            id="enc_type"
            className="border border-[#A5D6FF] bg-[#d0f1ff] text-sm rounded-lg focus:outline-none block w-full p-2.5 "
            onChange={handleSelectionChange}
          >
            <option selected value="">
              Choose Encryption Method
            </option>
            <option value="FHE">Fully Homomorphic Encryption [FHE]</option>
            <option value="AES">AES Dual Symmetric Key Encryption</option>
          </select>
        </div>
        <div>
          <label
            htmlFor="upload_file"
            className="block mb-2 text-md font-normal"
          >
            Upload Encrypted Data
          </label>
          <p className=" text-[14px] mb-2">
            Note: Encrypt Data using CLI tool before uploading{" "}
            <a href="#" className="text-blue underline">
              [docs]
            </a>
          </p>
          <input
            className="block w-full text-sm border border-[#A5D6FF] rounded-md cursor-pointer focus:outline-none bg-[#d0f1ff] p-2 mb-4"
            id="file_upload"
            type="file"
            onChange={handleFileChange}
          />
        </div>
        <div>
          <label
            htmlFor="description"
            className="block mb-2 text-md font-normal"
          >
            Description
          </label>
          <textarea
            id="description"
            className="border border-[#A5D6FF] bg-[#d0f1ff] text-sm rounded-lg focus:outline-none block w-full p-2.5 "
            onChange={handleDesChange}
            placeholder="Data Description"
          />
        </div>
        <button
          type="submit"
          className="p-3 bottom-1 border-[#4aa9f8] bg-[#d0f1ff] min-w-[150px] rounded-lg focus:outline-none mt-5 disabled:bg-gray-300 mb-2"
          disabled={!isConnected || !file || status != 0}
        >
          {status == 0 && (
            <>
              Store{" "}
              {!isConnected || !file
                ? "(disabled, Connect Wallet and Upload File)"
                : ""}
            </>
          )}
          {status == 1 && "Loading ... "}
          {status == 2 && "✅ Data Stored and ready for compute"}
        </button>
        <div>
          {" "}
          {ipfsHash !== "" && (
            <>
              Backup data storage:{" "}
              <a
                href={`https://files.lighthouse.storage/viewFile/${ipfsHash}`}
                target="_blank"
              >
                {ipfsHash}
              </a>
            </>
          )}
        </div>
      </form>
    </div>
  );
};

export default page;

function parseResponse(responseText: string) {
  // Use a regular expression to extract the Hash value
  const hashMatch = responseText.match(/Hash: "([^"]+)"/);
  if (hashMatch && hashMatch[1]) {
    return hashMatch[1];
  } else {
    throw new Error("Hash not found in the response");
  }
}
