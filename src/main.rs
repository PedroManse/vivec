use vivec::*;

fn main() {
    // cria uma lita com esses valores:
    let ppl = vec!["Hi0", "Hello1", "Goodday2", "Morning3", "YALP4"];
    // cria a lista ordenada com os valores
    let mut ppl = ViVec::from(ppl);
    ppl.unlink(2); // deleta Goodday2
    println!("{ppl:?}");
    ppl.append("Fuck2");

    println!("leitura simples");
    for p in ppl.straight_iter() {
        println!("{p:?}");
    }
    println!("leitura ordenada");
    for p in ppl.ordered_iter() {
        println!("{p:?}");
    }
}
