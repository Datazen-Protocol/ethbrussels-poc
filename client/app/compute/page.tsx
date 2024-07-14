"use client";
import React, { useEffect, useState } from "react";
import ComputePopup from "../components/ComputePopup";
interface DataItem {
  key: string;
  description: string;
  file_id: string;
}

const Page: React.FC = () => {
  const [data, setData] = useState<Record<string, DataItem[]>>({});

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch("http://localhost:8000/alldata/");
        const result = await response.json();
        setData(result);
      } catch (error) {
        console.error("Error fetching data:", error);
      }
    };

    fetchData();
  }, []);

  return (
    <div className="flex flex-col items-center content-center bg-transparent py-16 max-w-[80%] mx-auto">
      {Object.keys(data).map((address) => (
        <div
          key={address}
          className="bg-[#d0f1ff] shadow-lg rounded-lg p-6 w-full mb-8"
        >
          <h2 className="text-2xl font-semibold mb-4">Data Owner: {address}</h2>
          <div className="border-2 p-2 rounded-full border-green-400 w-max mb-5">
            Node Online
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 min-h-[160px]">
            {data[address]?.map((item, index) => (
              <div
                key={index}
                className="flex flex-col items-start space-y-5 border border-[#A5D6FF] p-4 rounded-lg shadow-sm"
              >
                <p className="text-lg">
                  <strong>Description:</strong> {item.description}
                </p>
                <p className="text-lg">
                  <strong>File ID:</strong> {item.file_id}
                </p>
                <ComputePopup address={address} file_id={item.file_id} />
              </div>
            )) || <p>No data available for this address.</p>}
          </div>
        </div>
      ))}
    </div>
  );
};

export default Page;
