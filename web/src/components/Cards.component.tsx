import * as mc from "@mantine/core";
import * as react from "react";
import { useSessionControl } from "../context/SessionControlProvider";

const CARD_VALUES = ["0", "1", "2", "3", "5", "8", "13", "20", "40", "60", "100", "?", "☕"]

export interface CardsProps {
  cardValues?: string[];
  visible?: boolean;
}

export default function Cards(props: CardsProps) {
  const cardValues = props.cardValues ?? CARD_VALUES;
  const visible = props.visible ?? true;
  if (!visible) {
    return <></>;
  }
  const sessionControl = useSessionControl();
  const [selected, setSelected] = react.useState<string | undefined>(undefined);
  const buttons: react.ReactNode[] = [];
  cardValues.forEach((cardValue) => {
    const onClick = () => {
      setSelected(cardValue);
      sessionControl.setPoints(cardValue);
    };
    const variant = selected === cardValue ? "filled" : "outline";
    buttons.push(<mc.Button key={cardValue} variant={variant} onClick={onClick}>{cardValue}</mc.Button>);
  });
  return (
    <mc.Group>{buttons}</mc.Group>
  )
}
