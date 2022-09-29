# Categorizador

Um categorizador de produtos que tenta ser leve, precisa de bastante otimização na hora de encontrar
os tokens dentro do nome do produto mas funciona que é uma beleza.
É basicamente uma rede neural que trabalha com pares de palavras ao invés de com palavras individuais.

Isso faz com que seja impossível de se categorizar com apenas uma palavra!

## Como rodar

Basta compilar o código, tendo Rust instalado no computador, com um `cargo build --release` e depois
rodar o binário usando o nome de um produto. Após alguns segundos de procurando as palavras certas
no vocabulário ele vai dizer a categoria que ele acha mais provável pra classificar seu produto
dentre as categorias do Mercado Lívre juntamente com a confinça que ele acha que é esta categoria.

Para rodar também vai precisar do `vocab.bin` e da pasta weights ao lado do binário.

Alguns exemploes do resultados da rede neural são:

| Nome do produto | Resultado |
|       ---       |    ---    |
|   galaxy a32    | ("celulares e telefones celulares e smartphones", 1.0) | 
| Pasta Bolsa Executiva Para Notebook Laptop Macbook Casual | ("calcados, roupas e bolsas malas e bolsas", 0.24538739) |
| pasta térmica | ("informatica limpeza de pcs", 2.0) |
| Guarda roupa casal 4 portas | ("casa, moveis e decoracao organizacao para casa", 0.9679922) |

## Como foi treinado?

Essa "rede neural" foi treinada usando mais ou menos 3 milhões de produtos com sua respectiva categoria
do Mercado Livre (que já tem um categorizador desses, e bem melhor por sinal), contando primeiramente
quantas vezes pares de palavras que ficam vizinhas umas às outras ocorrem em relação cada categoria.

Isso acaba sendo um tanto eficiente pra poder classificar os produtos sem ter que ter usado
muita computação pra treinar.

## Dataset

Os dados utilizados estão salvos no árquivo `training-data.bin`. Para lê-lo você só precisar da bibliotéca
Savefile e usar a função `load_file` com seu tipo sendo uma Vec da struct abaixo:

```rust
// Vec<Sample>
#[derive(Debug, Clone, Savefile)]
struct Sample {
    product_name: String,
    category: Category,
}

#[derive(Debug, Clone, Savefile)]
struct Category {
    parent: String,
    category: String,
    url: String,
}
```
Essa não é a forma mais eficiente de se guardar esses dados, (como discuti com o dono do Savefile),
mas é suficiente.

## Considerações finais

Se divirta usando! Se quiser dar uma otimizada na velocidade do código dando um pull request aí 
eu aceito.
