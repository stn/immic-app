import { useEffect, useState } from "react";
import { Link } from "react-router-dom";

import {
  listDates,
} from "@/lib/api";

function DailyPage() {
  const [dates, setDates] = useState<string[]>([]);

  useEffect(() => {
    let isMounted = true;

    (async () => {
      let ds = await listDates();
      if (isMounted) {
        setDates(ds);
      }
    })();

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <div>
      { dates && dates.map((d) => (
        <div key={d}
          className="text-5xl font-semibold my-6"
        >
          <Link to={`/${d.slice(0, 4)}/${d.slice(4, 6)}/${d.slice(6, 8)}`}>
            {d.slice(0, 4)}/{d.slice(4, 6)}/{d.slice(6, 8)}
          </Link>
        </div>
      ))}
    </div>
  );
}

export default DailyPage;
