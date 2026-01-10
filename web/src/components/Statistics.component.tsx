import * as mc from "@mantine/core";
import { useSessionControl } from "../context/SessionControlProvider";

export default function Statistics() {
  const sessionControl = useSessionControl();
  const sessionState = sessionControl.sessionState;
  if (!sessionControl.revealPoints || sessionState === undefined) {
    return <></>;
  }

  // Determine mean vote, low vote and high vote.
  let meanVote = 0.0;
  let lowVote = Number.POSITIVE_INFINITY;
  let highVote = Number.NEGATIVE_INFINITY;
  let numUsers = 0;
  let votesExcluded = 0;
  for (const uid in sessionState.users) {
    const user = sessionState.users[uid];
    if (user.isSpectator) {
      continue;
    }
    const points = Number(user.points);
    if (!Number.isFinite(points)) {
      votesExcluded += 1;
      continue;
    }
    meanVote += points; 
    numUsers += 1;
    if (points < lowVote) {
      lowVote = points;
    }
    if (points > highVote) {
      highVote = points;
    }
  }
  meanVote /= numUsers;

  // Determine low and high voters.
  const lowVoters = [];
  const highVoters = [];
  for (const uid in sessionState.users) {
    const user = sessionState.users[uid];
    if (user.points == lowVote) {
      lowVoters.push(user.name);
    }
    if (user.points == highVote) {
      highVoters.push(user.name);
    }
  }

  const integerIfFinite = (number: number): string => Number.isFinite(number) ? number.toFixed() : "?";

  return (
    <mc.Table variant="vertical">
      <mc.Table.Tbody>
        <mc.Table.Tr>
          <mc.Table.Th w={120}>Mean Vote</mc.Table.Th>
          <mc.Table.Td w={60}>{integerIfFinite(meanVote)}</mc.Table.Td>
          <mc.Table.Td>{votesExcluded > 0 ? ` (${votesExcluded} votes excluded)` : ""}</mc.Table.Td>
        </mc.Table.Tr>
        <mc.Table.Tr>
          <mc.Table.Th>Low Vote</mc.Table.Th>
          <mc.Table.Td>{integerIfFinite(lowVote)}</mc.Table.Td>
          <mc.Table.Td>{lowVoters.length > 0 ? `(${lowVoters.join(", ")})` : ""}</mc.Table.Td>
        </mc.Table.Tr>
        <mc.Table.Tr>
          <mc.Table.Th>High Vote</mc.Table.Th>
          <mc.Table.Td>{integerIfFinite(highVote)}</mc.Table.Td>
          <mc.Table.Td>{highVoters.length > 0 ? `(${highVoters.join(", ")})` : ""}</mc.Table.Td>
        </mc.Table.Tr>
      </mc.Table.Tbody>
    </mc.Table>
  )
}
